from pathlib import Path
W=Path(__file__).parent

def replace_run(s, transform):
 a=s.index('    fn parse_html_block_run(');b=s.index('    /// Parse a single HTML block line',a)
 return s[:a]+transform(s[a:b])+s[b:]

def variant(name,files):
 if name=='baseline':return files
 if name=='production':
  for p in (W/'production-source').rglob('*.rs'):
   files[str(p.relative_to(W/'production-source'))]=p.read_text()
  return files
 s=files['src/block/parser.rs']
 for part in name.split('+'):
  if part=='probe':
   s=s.replace('    fn parse_html_block_run(', '    #[inline(never)]\n    fn parse_html_block_run(',1)
  elif part=='local-indent':
   def change(r):
    return r.replace('            self.skip_indent();', '''            let mut col = 0;
            let mut count = 0;
            for &byte in self.cursor.remaining_slice() {
                match byte {
                    b' ' => col += 1,
                    b'\\t' => col = Self::tab_column(col),
                    _ => break,
                }
                count += 1;
            }
            self.current_col = col;
            self.cursor = Cursor::new_at(self.input, start + count);''',1)
   s=replace_run(s,change)
  elif part=='specialize':
   def change(r):
    r=r.replace('    fn parse_html_block_run(&mut self, events: &mut Vec<BlockEvent>) {', '''    fn parse_html_block_run(&mut self, events: &mut Vec<BlockEvent>) {
        if matches!(self.html_block, Some(HtmlBlockKind::Type6 | HtmlBlockKind::Type7)) {
            self.parse_html_block_run_kind::<true>(events);
        } else {
            self.parse_html_block_run_kind::<false>(events);
        }
    }

    fn parse_html_block_run_kind<const BLANK: bool>(&mut self, events: &mut Vec<BlockEvent>) {''',1)
    r=r.replace('if matches!(kind, HtmlBlockKind::Type6 | HtmlBlockKind::Type7)','if BLANK')
    r=r.replace('if self.html_block_ends(kind,','if !BLANK && self.html_block_ends(kind,')
    return r
   s=replace_run(s,change)
  elif part=='defer-indent':
   def change(r):
    r=r.replace('            self.skip_indent();', '''            // Probe: only blank detection needs indentation during scanning.
            let blank_ended = matches!(kind, HtmlBlockKind::Type6 | HtmlBlockKind::Type7);
            if blank_ended {
                self.cursor.skip_whitespace();
            }''',1)
    r=r.replace('            if self.html_block_ends(kind,', '            if self.html_block_ends(kind,')
    r=r.replace('''                events.push(BlockEvent::HtmlBlockEnd);
            }
        }
    }''', '''                events.push(BlockEvent::HtmlBlockEnd);
            }
            if self.html_block.is_none() || self.cursor.is_eof() {
                let end_cursor = self.cursor;
                self.cursor = Cursor::new_at(self.input, start);
                self.skip_indent();
                self.cursor = end_cursor;
            }
        }
    }''')
    return r
   s=replace_run(s,change)
  elif part in ['newline16','newline32','newline64','newline128','newline-all']:
   def change(r):
    old="            let distance = memchr::memchr(b'\\n', rest).unwrap_or(rest.len());"
    if part=='newline-all':
     new="            let distance = crate::byte_search::ByteSet::new(b\"\\n\").find(rest).unwrap_or(rest.len());"
    else:
     size=int(part.removeprefix('newline'))
     new=f'''            let distance = if rest.len() >= {size} {{
                crate::byte_search::ByteSet::new(b"\\n").find(&rest[..{size}])
                    .unwrap_or_else(|| {size} + memchr::memchr(b'\\n', &rest[{size}..]).unwrap_or(rest.len() - {size}))
            }} else {{
                memchr::memchr(b'\\n', rest).unwrap_or(rest.len())
            }};'''
    assert old in r
    return r.replace(old,new,1)
   s=replace_run(s,change)
  elif part=='word16':
   def change(r):
    old="            let distance = memchr::memchr(b'\\n', rest).unwrap_or(rest.len());"
    new='''            let distance = if rest.len() >= 16 {
                let first = u64::from_le_bytes(rest[..8].try_into().unwrap()) ^ 0x0a0a0a0a0a0a0a0a;
                let mask = first.wrapping_sub(0x0101010101010101) & !first & 0x8080808080808080;
                if mask != 0 {
                    (mask.trailing_zeros() / 8) as usize
                } else {
                    let second = u64::from_le_bytes(rest[8..16].try_into().unwrap()) ^ 0x0a0a0a0a0a0a0a0a;
                    let mask = second.wrapping_sub(0x0101010101010101) & !second & 0x8080808080808080;
                    if mask != 0 { 8 + (mask.trailing_zeros() / 8) as usize }
                    else { 16 + memchr::memchr(b'\\n', &rest[16..]).unwrap_or(rest.len() - 16) }
                }
            } else { memchr::memchr(b'\\n', rest).unwrap_or(rest.len()) };'''
    assert old in r
    return r.replace(old,new,1)
   s=replace_run(s,change)
  elif part=='single-byte':
   key='src/byte_search.rs'
   files[key]=files[key].replace('            if self.nibble_valid {', '            if N == 1 {\n                vceqq_u8(v, vdupq_n_u8(self.bytes[0]))\n            } else if self.nibble_valid {', 1)
  elif part=='const-set':
   old='crate::byte_search::ByteSet::new(b"\\n")'
   assert old in s
   s=s.replace(old,'const { '+old+' }')
  elif part=='stable-kind':
   def change(r):
    r=r.replace('        while self.html_block.is_some() && !self.cursor.is_eof() {', '        let kind = self.html_block.unwrap();\n        while !self.cursor.is_eof() {',1)
    r=r.replace('            let kind = self.html_block.unwrap();\n','',1)
    r=r.replace('                continue;','                return;',1)
    r=r.replace('                events.push(BlockEvent::HtmlBlockEnd);','                events.push(BlockEvent::HtmlBlockEnd);\n                return;',1)
    return r
   s=replace_run(s,change)
  elif part in ['bounded16','bounded32']:
   def change(r):
    old="            let distance = memchr::memchr(b'\\n', rest).unwrap_or(rest.len());"
    size=16 if part=='bounded16' else 32
    new=f'''            let distance = if rest.len() >= {size} {{
                memchr::memchr(b'\\n', &rest[..{size}])
                    .unwrap_or_else(|| {size} + memchr::memchr(b'\\n', &rest[{size}..]).unwrap_or(rest.len() - {size}))
            }} else {{
                memchr::memchr(b'\\n', rest).unwrap_or(rest.len())
            }};'''
    assert old in r
    return r.replace(old,new,1)
   s=replace_run(s,change)
  elif part=='prefix':
   s=s.replace('''        let line = self.current_line_slice();
        if line.is_empty() {
            return None;
        }

        // Type 1''','''        let line = self.cursor.remaining_slice();
        if line.first() != Some(&b'<') {
            return None;
        }

        // Type 1''',1)
   s=s.replace('''        if let Some((_name, tag_end)) = self.parse_html_tag(line)''','''        let line = self.current_line_slice();
        if let Some((_name, tag_end)) = self.parse_html_tag(line)''',1)
  else:raise ValueError(part)
 files['src/block/parser.rs']=s
 return files
