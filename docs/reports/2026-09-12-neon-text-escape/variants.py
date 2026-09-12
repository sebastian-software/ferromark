def variant(name,files):
 if name=='baseline':return files
 p='src/escape.rs';s=files[p]
 start=s.index('    if input.len() <= SHORT_SCAN_MAX {',s.index('pub(crate) fn first_text_escape'))
 end=s.index('\n}',start);original=s[start:end]
 if name=='bulk-inline-neon':
  files=variant('bulk-neon',files)
  gate='all(target_arch = "aarch64", target_feature = "neon")'
  files[p]=files[p].replace('#[inline]\npub fn escape_text_into',f'#[cfg_attr({gate}, inline(always))]\n#[cfg_attr(not({gate}), inline)]\npub fn escape_text_into',1)
  return files
 if name=='bulk-neon':
  first=s.index('    let mut start = 0usize;', s.index('pub fn escape_text_into'))
  last=s.index('\n}',first)
  old=s[first:last]
  gate='all(target_arch = "aarch64", target_feature = "neon")'
  branch=f'    #[cfg({gate})]\n    if input.len() > SHORT_SCAN_MAX {{\n        escape_text_long_neon(out, input);\n        return;\n    }}\n\n'
  body=old.replace('first_text_escape(&input[start..])','TEXT_SPECIALS.find(&input[start..])')
  helper=f'\n\n#[cfg({gate})]\n#[inline(never)]\nfn escape_text_long_neon(out: &mut Vec<u8>, input: &[u8]) {{\n'+body+'\n}'
  files[p]=s[:first]+branch+old+s[last:last+2]+helper+s[last+2:]
  return files
 if name=='tail-neon':
  first=s.index('    let mut start = 0usize;', s.index('pub fn escape_text_into'))
  last=s.index('\n}',first)
  old=s[first:last]
  tail=old.replace('    let mut start = 0usize;\n    while let Some(rel) = first_text_escape(&input[start..]) {','    let mut start = 0usize;\n    let mut next = first_text_escape(input);\n    while let Some(rel) = next {')
  tail=tail.replace('        start = pos + 1;', '        start = pos + 1;\n        next = TEXT_SPECIALS.find(&input[start..]);')
  gate='all(target_arch = "aarch64", target_feature = "neon")'
  files[p]=s[:first]+f'    #[cfg({gate})]\n    {{\n'+tail+'\n    }\n'+f'    #[cfg(not({gate}))]\n    {{\n'+old+'\n    }'+s[last:]
  return files
 if name=='outlined-neon':
  gate='all(target_arch = "aarch64", target_feature = "neon")'
  short_end=original.index("    let a = memchr3")
  body=original[:short_end]+f'    #[cfg({gate})]\n    {{ first_text_escape_long(input) }}\n    #[cfg(not({gate}))]\n    {{\n'+original[short_end:]+'\n    }'
  helper=f'\n\n#[cfg({gate})]\n#[inline(never)]\nfn first_text_escape_long(input: &[u8]) -> Option<usize> {{ TEXT_SPECIALS.find(input) }}'
  files[p]=s[:start]+body+s[end:end+2]+helper+s[end+2:]
  return files
 if name=='single-neon':
  body='    TEXT_SPECIALS.find(input)'
 elif name.startswith('prefix-'):
  size=int(name.split('-')[1])
  body=f'''    const PREFIX: usize = {size};
    if input.len() <= PREFIX {{ return TEXT_SPECIALS.find(input); }}
    if let Some(at) = TEXT_SPECIALS.find(&input[..PREFIX]) {{ return Some(at); }}
    let input = &input[PREFIX..];
    let a = memchr3(b'<', b'>', b'&', input);
    let limit = a.unwrap_or(input.len());
    memchr(b'"', &input[..limit]).or(a).map(|at| PREFIX + at)'''
 elif name=='direct-neon':
  from pathlib import Path
  assert (Path(__file__).parent/'direct-neon-source/src/escape.rs').is_file(), 'direct-neon source must exist'
  return {str(p.relative_to(Path(__file__).parent/'direct-neon-source')):p.read_text() for p in (Path(__file__).parent/'direct-neon-source/src').rglob('*.rs')}
 else:raise ValueError(name)
 gate='all(target_arch = "aarch64", target_feature = "neon")'
 s=s[:start]+f'    #[cfg({gate})]\n    {{\n'+body+'\n    }\n'+f'    #[cfg(not({gate}))]\n    {{\n'+original+'\n    }'+s[end:];files[p]=s;return files
