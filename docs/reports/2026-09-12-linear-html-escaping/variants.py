from pathlib import Path
W=Path(__file__).parent

def variant(name,files):
 if name=='baseline':return files
 if name=='final':
  assert (W/'final-source/src/escape.rs').is_file()
  return {str(p.relative_to(W/'final-source')):p.read_text() for p in (W/'final-source/src').rglob('*.rs')}
 if name=='early-short':
  files=variant('final',files)
  p='src/escape.rs';s=files[p]
  first=s.index('    let prefix_len =',s.index('fn first_escape<const'))
  last=s.index('\n}',first)
  s=s[:first]+'    if input.len() <= SHORT_SCAN_MAX {\n        return first_escape_in_set::<ATTR>(input);\n    }\n    if let Some(at) = first_escape_in_set::<ATTR>(&input[..SHORT_SCAN_MAX]) {\n        return Some(at);\n    }\n    first_escape_long::<ATTR>(&input[SHORT_SCAN_MAX..]).map(|at| SHORT_SCAN_MAX + at)'+s[last:]
  files[p]=s;return files
 if name.startswith('threshold-'):
  threshold=int(name.split('-')[-1]);p='src/escape.rs';s=files[p]
  final=variant('final',files.copy())[p]
  first=final.index('// Keep the long-search loop')
  last=final.index('// Escape policy stays here')
  helper=final[first:last].replace('first_escape_in_set','first_escape_short')
  helper=helper.replace('        #[cfg(test)]\n        tests::record_long_search(input.len(), found);\n','')
  helper=helper.replace('            #[cfg(test)]\n            tests::record_long_search(chunk.len(), common);\n','')
  for attr in [False,True]:
   fname='first_attr_escape' if attr else 'first_text_escape'
   first=s.index('    let a = memchr3',s.index('fn '+fname+'('))
   s=s[:first]+f'    if input.len() > {threshold} {{ return first_escape_long::<{str(attr).lower()}>(input); }}\n'+s[first:]
  files[p]=s+'\n'+helper;return files
 if name.startswith('bulk-'):
  threshold=int(name.split('-')[-1]);p='src/escape.rs';s=files[p]
  final=variant('final',files.copy())[p]
  first=final.index('// Keep the long-search loop')
  last=final.index('// Escape policy stays here')
  helper=final[first:last].replace('first_escape_in_set','first_escape_short').replace('#[inline(never)]','#[inline]')
  helper=helper.replace('        #[cfg(test)]\n        tests::record_long_search(input.len(), found);\n','')
  helper=helper.replace('            #[cfg(test)]\n            tests::record_long_search(chunk.len(), common);\n','')
  for attr in [False,True]:
   fname='escape_full_into' if attr else 'escape_text_into'
   first=s.index('    let mut start = 0usize;',s.index('pub fn '+fname+'('))
   s=s[:first]+f'    if input.len() > {threshold} {{ escape_linear::<{str(attr).lower()}>(out, input); return; }}\n'+s[first:]
  s+='\n'+helper+'\n#[inline(never)]\nfn escape_linear<const ATTR: bool>(out: &mut Vec<u8>, input: &[u8]) {\n    let mut start = 0;\n    while let Some(rel) = first_escape_long::<ATTR>(&input[start..]) {\n        let at = start + rel;\n        out.extend_from_slice(&input[start..at]);\n        if ATTR { push_attr_escape(out, input[at]); } else { push_text_escape(out, input[at]); }\n        start = at + 1;\n    }\n    out.extend_from_slice(&input[start..]);\n}\n'
  files[p]=s;return files
 if name.startswith('wide-'):
  files=variant('production',files);p='src/escape.rs'
  files[p]=files[p].replace('let mut width = SHORT_SCAN_MAX * 2;',f'let mut width = {int(name.split("-")[-1])};')
  return files
 if name=='hybrid-tail':
  files=variant('final',files);p='src/escape.rs';s=files[p]
  s=s.replace('#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]\nuse memchr::{memchr2, memchr3};','use memchr::{memchr2, memchr3};')
  s=s.replace('        found\n    }\n    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]\n    {','        return found;\n    }\n    {')
  first=s.index('    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]',s.index('fn first_escape_long'))
  s=s[:first]+s[first:].replace('    {\n        let found =','    if input.len() >= 256 {\n        let found =',1)
  files[p]=s;return files
 if name=='linear':
  return {str(p.relative_to(W/'linear-source')):p.read_text() for p in (W/'linear-source/src').rglob('*.rs')}
 if name=='production':
  assert (W/'production-source/src/escape.rs').is_file()
  return {str(p.relative_to(W/'production-source')):p.read_text() for p in (W/'production-source/src').rglob('*.rs')}
 p='src/escape.rs';s=files[p]
 if name=='neon-tail':
  files=variant('production',files);s=files[p]
  first=s.index('    let mut start = 0;',s.index('fn first_escape_long'))
  last=s.index('\n}',first);body=s[first:last]
  gate='all(target_arch = "aarch64", target_feature = "neon")'
  s=s[:first]+f'''    #[cfg({gate})]
    {{
        let found = first_escape_short::<ATTR>(input);
        #[cfg(test)]
        tests::record_common_search(input.len(), found);
        found
    }}
    #[cfg(not({gate}))]
    {{
'''+body+'\n    }'+s[last:]
  s=s.replace('use memchr::{memchr, memchr2, memchr3};', 'use memchr::memchr;\n#[cfg(not(all(target_arch = \"aarch64\", target_feature = \"neon\")))]\nuse memchr::{memchr2, memchr3};')
  s=s.replace('COMMON_SEARCH_BYTES','LONG_SEARCH_BYTES').replace('record_common_search','record_long_search')
  s=s.replace('// Count the logical prefix inspected by the three-byte search. The known\n    // quote-only fixtures require it to read the complete supplied range.', '// Count the logical prefix inspected by each long-search window. This is\n    // compiled out of release builds and avoids timing thresholds in tests.')
  files[p]=s;return files
 if name in ['outlined-growth','outlined-prefix']:
  prefix=name=='outlined-prefix'
  helpers=[]
  for attr in [False,True]:
   fname='first_attr_escape' if attr else 'first_text_escape'
   first=s.index('    if input.len() <= SHORT_SCAN_MAX',s.index('fn '+fname+'('));last=s.index('\n}',first)
   short=f'first_escape_short::<{str(attr).lower()}>'
   if prefix:
    body=f'''    let prefix_len = input.len().min(SHORT_SCAN_MAX);
    if let Some(at) = {short}(&input[..prefix_len]) {{ return Some(at); }}
    if input.len() > prefix_len {{
        {fname}_long(&input[prefix_len..]).map(|at| prefix_len + at)
    }} else {{ None }}'''
   else:body=f'''    if input.len() <= SHORT_SCAN_MAX {{ return {short}(input); }}
    {fname}_long(input)'''
   quotes="memchr2(b'\"', b'\\\'', &chunk[..limit])" if attr else "memchr(b'\"', &chunk[..limit])"
   helpers.append(f'''
#[inline(never)]
fn {fname}_long(input: &[u8]) -> Option<usize> {{
    let mut start = 0;
    let mut width = SHORT_SCAN_MAX * {2 if prefix else 1};
    while start < input.len() {{
        let len = width.min(input.len() - start);
        let chunk = &input[start..start + len];
        let a = memchr3(b'<', b'>', b'&', chunk);
        let limit = a.unwrap_or(chunk.len());
        if let Some(at) = {quotes}.or(a) {{ return Some(start + at); }}
        start += len;
        width = width.saturating_mul(2);
    }}
    None
}}
''')
   s=s[:first]+body+s[last:]
  files[p]=s+''.join(helpers);return files
 if name.startswith('grow-'):
  prefix=name=='grow-prefix';size=128
  for attr in [False,True]:
   fname='first_attr_escape' if attr else 'first_text_escape'
   first=s.index('    let a = memchr3',s.index('fn '+fname+'('));last=s.index('\n}',first)
   quotes="memchr2(b'\"', b'\\\'', &chunk[..limit])" if attr else "memchr(b'\"', &chunk[..limit])"
   body=''
   if prefix:
    body=f'''    if let Some(at) = first_escape_short::<{str(attr).lower()}>(&input[..SHORT_SCAN_MAX]) {{ return Some(at); }}
    let mut start = SHORT_SCAN_MAX;
    let mut width = SHORT_SCAN_MAX * 2;
'''
   else:body='    let mut start = 0;\n    let mut width = SHORT_SCAN_MAX;\n'
   body+=f'''    while start < input.len() {{
        let len = width.min(input.len() - start);
        let chunk = &input[start..start + len];
        let a = memchr3(b'<', b'>', b'&', chunk);
        let limit = a.unwrap_or(chunk.len());
        if let Some(at) = {quotes}.or(a) {{ return Some(start + at); }}
        start += len;
        width = width.saturating_mul(2);
    }}
    None'''
   s=s[:first]+body+s[last:]
 elif name=='cached-long':
  for attr in [False,True]:
   fname='escape_full_into' if attr else 'escape_text_into'
   first=s.index('    let mut start = 0usize;',s.index('pub fn '+fname+'('))
   s=s[:first]+f'''    if input.len() > SHORT_SCAN_MAX {{
        escape_cached::<{str(attr).lower()}>(out, input);
        return;
    }}

'''+s[first:]
  s+='''
#[inline(never)]
fn escape_cached<const ATTR: bool>(out: &mut Vec<u8>, input: &[u8]) {
    let mut start = 0;
    let mut common = memchr3(b'<', b'>', b'&', input).unwrap_or(input.len());
    let mut quote = if ATTR { memchr2(b'"', b'\\\'', input) } else { memchr(b'"', input) }.unwrap_or(input.len());
    loop {
        let at = common.min(quote);
        out.extend_from_slice(&input[start..at]);
        if at == input.len() { break; }
        if ATTR { push_attr_escape(out, input[at]); } else { push_text_escape(out, input[at]); }
        start = at + 1;
        if at == common { common = memchr3(b'<', b'>', b'&', &input[start..]).map_or(input.len(), |i| start + i); }
        else { quote = if ATTR { memchr2(b'"', b'\\\'', &input[start..]) } else { memchr(b'"', &input[start..]) }.map_or(input.len(), |i| start + i); }
    }
}
'''
 else:raise ValueError(name)
 files[p]=s;return files
