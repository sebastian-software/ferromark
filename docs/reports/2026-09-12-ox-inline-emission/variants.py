def once(s,a,b):
 assert s.count(a)==1,(a[:100],s.count(a))
 return s.replace(a,b,1)

def code_one(s):
 a=s.index('            emit_points.push(EmitPoint {',s.index('// Add code span events'))
 b=s.index('\n        }',a)
 s=s[:a]+'''            let (content_start, content_end) = span.content_range();
            emit_points.push(EmitPoint {
                pos: span.opener_pos,
                kind: EmitKind::CodeSpan { content_start, content_end },
                end: span.closer_end,
            });'''+s[b:]
 a=s.index('                EmitKind::CodeSpanStart => {');b=s.index('                EmitKind::EmphasisStart => {',a)
 old=s[a:b];body=old[old.index('                    // Emit code content'):];body=body.replace('let mut start = point.pos as usize;','let mut start = content_start as usize;').replace('let mut end = end as usize;','let mut end = content_end as usize;').replace('skip_until = end as u32;','skip_until = point.end;')
 s=s[:a]+'                EmitKind::CodeSpan { content_start, content_end } => {\n'+body+s[b:]
 s=once(s,'    CodeSpanStart,\n    CodeSpanEnd,\n    CodeContent(u32), // end position','    CodeSpan { content_start: u32, content_end: u32 },')
 s=once(s,'EmitKind::CodeSpanEnd\n                        | EmitKind::StrongEnd','EmitKind::StrongEnd')
 return s

def code_two(s):
 s=code_one(s)
 s=once(s,"""                kind: EmitKind::CodeSpan { content_start, content_end },
                end: span.closer_end,
            });""","""                kind: EmitKind::CodeSpan { content_start, content_end },
                end: span.closer_pos,
            });
            emit_points.push(EmitPoint {
                pos: span.closer_pos,
                kind: EmitKind::CodeSpanEnd,
                end: span.closer_end,
            });""")
 s=once(s,'                    skip_until = point.end;\n                }\n                EmitKind::EmphasisStart', '                    skip_until = end as u32;\n                }\n                EmitKind::CodeSpanEnd => { skip_until = point.end; }\n                EmitKind::EmphasisStart')
 s=once(s,'    CodeSpan { content_start: u32, content_end: u32 },','    CodeSpan { content_start: u32, content_end: u32 },\n    CodeSpanEnd,')
 s=once(s,'                    EmitKind::StrongEnd','                    EmitKind::CodeSpanEnd | EmitKind::StrongEnd')
 return s

def code_safe(s):
 old=s
 s=code_two(s)
 a=old.index('            emit_points.push(EmitPoint {',old.index('// Add code span events'));b=old.index('\n        }',a)
 fallback=old[a:b]
 s=once(s,'        // Add code span events (filter out spans whose opener is inside an autolink)',"""        let fuse_code = resolved_links.is_empty() && resolved_ref_links.is_empty()
            && math_spans.is_empty() && footnote_refs.is_empty() && inline_footnotes.is_empty();
        // Add code span events (filter out spans whose opener is inside an autolink)""")
 a=s.index('            let (content_start, content_end) = span.content_range();',s.index('// Add code span events'));b=s.index('\n        }',a)
 s=s[:a]+'            if fuse_code {\n'+s[a:b]+'\n            } else {\n'+fallback+'\n            }'+s[b:]
 a=old.index('                EmitKind::CodeSpanStart => {');b=old.index('                EmitKind::EmphasisStart => {',a)
 legacy=old[a:b];start=legacy.index('                EmitKind::CodeSpanEnd => {');end=legacy.index('                EmitKind::CodeContent(',start)
 legacy=legacy[:start]+legacy[end:]
 s=once(s,'                EmitKind::EmphasisStart => {',legacy+'                EmitKind::EmphasisStart => {')
 s=once(s,'    CodeSpan { content_start: u32, content_end: u32 },','    CodeSpan { content_start: u32, content_end: u32 },\n    CodeSpanStart,\n    CodeContent(u32),')
 return s

def compact_links(s):
 a=s.index('    fn emit_events(');p,e=s[:a],s[a:]
 e=once(e,'        for link in resolved_links {','        for (link_index, link) in resolved_links.iter().enumerate() {')
 for k in ['LinkStart','ImageStart']:
  e=once(e,'''kind: EmitKind::'''+k+''' {
                        url_start: link.url_start,
                        url_end: link.url_end,
                        title_start: link.title_start,
                        title_end: link.title_end,
                    },''','kind: EmitKind::'+k+' { link_index: link_index as u32 },')
  e=once(e,'''                EmitKind::'''+k+''' {
                    url_start,
                    url_end,
                    title_start,
                    title_end,
                } => {''','''                EmitKind::'''+k+''' { link_index } => {
                    let link = &resolved_links[link_index as usize];
                    let (url_start, url_end, title_start, title_end) =
                        (link.url_start, link.url_end, link.title_start, link.title_end);''')
  e=once(e,'''    '''+k+''' {
        url_start: u32,
        url_end: u32,
        title_start: Option<u32>,
        title_end: Option<u32>,
    },''','    '+k+' { link_index: u32 },')
 return p+e

def merge_marks(s):
 a=s.index('        // Sort by position (end events come after start events at same position)');b=s.index('\n\n',a)
 sort=s[a:b]
 s=s[:a]+'''        let (resolved_points, mark_points) = emit_points.split_at(mark_points_start);
        let mut left = resolved_points.iter().peekable();
        let mut right = mark_points.iter().peekable();
        let ordered_points = std::iter::from_fn(move || {
            match (left.peek(), right.peek()) {
                (Some(a), Some(b)) if emit_point_key(a) <= emit_point_key(b) => left.next(),
                (Some(_), Some(_)) => right.next(),
                (Some(_), None) => left.next(),
                (None, Some(_)) => right.next(),
                (None, None) => None,
            }
        });'''+s[b:]
 s=once(s,'        // Add backslash escapes and hard breaks', '        emit_points.sort_unstable_by_key(emit_point_key);\n        let mark_points_start = emit_points.len();\n\n        // Add backslash escapes and hard breaks')
 s=once(s,'        for point in emit_points.iter() {','        for point in ordered_points {')
 body=sort[sort.index('            ('):sort.rindex('        });')]
 helper='\n#[inline]\nfn emit_point_key(p: &EmitPoint) -> (u32, bool) {\n'+body+'}\n'
 s=once(s,'#[derive(Debug, Clone, Copy)]\nstruct EmitPoint',helper+'\n#[derive(Debug, Clone, Copy)]\nstruct EmitPoint')
 return s

def merge_slices(s):
 s=merge_marks(s)
 a=s.index('        let mut left = resolved_points.iter().peekable();');b=s.index('\n\n',a)
 s=s[:a]+"""        let mut left = resolved_points;
        let mut right = mark_points;
        let ordered_points = std::iter::from_fn(move || {
            if right.is_empty() || (!left.is_empty() && emit_point_key(&left[0]) <= emit_point_key(&right[0])) {
                let (point, rest) = left.split_first()?;
                left = rest;
                Some(point)
            } else {
                let (point, rest) = right.split_first()?;
                right = rest;
                Some(point)
            }
        });"""+s[b:]
 return s

def insert_marks(s):
 a=s.index('        // Sort by position (end events come after start events at same position)');b=s.index('\n\n',a)
 old=s[a:b];body=old[old.index('            ('):old.rindex('        });')]
 replacement="""        let mark_count = emit_points.len() - mark_points_start;
        if mark_points_start != 0 {
            if mark_count != 0 && mark_count <= 16 && emit_points.len() <= 128 {
                emit_points[..mark_points_start].sort_unstable_by_key(emit_point_key);
                for index in mark_points_start..emit_points.len() {
                    let point = emit_points[index];
                    let key = emit_point_key(&point);
                    let insertion = emit_points[..index].partition_point(|p| emit_point_key(p) <= key);
                    emit_points.copy_within(insertion..index, insertion + 1);
                    emit_points[insertion] = point;
                }
            } else {
                emit_points.sort_unstable_by_key(emit_point_key);
            }
        }"""
 s=s[:a]+replacement+s[b:]
 s=once(s,'        // Add backslash escapes and hard breaks','        let mark_points_start = emit_points.len();\n\n        // Add backslash escapes and hard breaks')
 helper='\n#[inline]\nfn emit_point_key(p: &EmitPoint) -> (u32, bool) {\n'+body+'}\n'
 s=once(s,'#[derive(Debug, Clone, Copy)]\nstruct EmitPoint',helper+'\n#[derive(Debug, Clone, Copy)]\nstruct EmitPoint')
 return s

def html_search(s):
 a=s.index('fn find_html_spans_into(');b=s.index('\nfn filter_html_spans_in_link_destinations',a);prefix,body,suffix=s[:a],s[a:b],s[b:]
 body=once(body,'    let len = text.len();\n','')
 body=once(body,"""    while pos < len {
        if text[pos] != b'<' {
            pos += 1;
            continue;
        }
""","""    while let Some(offset) = memchr(b'<', &text[pos..]) {
        pos += offset;
""")
 return prefix+body+suffix

def variant(name,files):
 if name=='baseline':return files
 if name=='production':
  from pathlib import Path
  files['src/inline/mod.rs']=(Path(__file__).parent/'production-inline.rs').read_text()
  return files
 s=files['src/inline/mod.rs']
 for part in name.split('+'):
  if part=='code-one':s=code_one(s)
  elif part=='code-two':s=code_two(s)
  elif part=='code-safe':s=code_safe(s)
  elif part=='compact-links':s=compact_links(s)
  elif part=='compact-all':
   from compact_variant import compact_all
   s=compact_all(s)
  elif part=='merge-marks':s=merge_marks(s)
  elif part=='merge-slices':s=merge_slices(s)
  elif part=='insert-marks':s=insert_marks(s)
  elif part=='html-search':s=html_search(s)
  elif part=='compact-code':
   from compact_variant import compact_code
   s=compact_code(s)
  else:raise ValueError(part)
 files['src/inline/mod.rs']=s
 return files
