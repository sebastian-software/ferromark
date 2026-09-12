RUN = '''
    /// Consume root HTML continuations without repeating general block dispatch.
    fn parse_html_block_run(&mut self, events: &mut Vec<BlockEvent>) {
        while self.html_block.is_some() && !self.cursor.is_eof() {
            self.current_line_start = self.cursor.offset();
            if self.options.definition_lists {
                self.definition_current_line_loose =
                    std::mem::take(&mut self.definition_blank_before_next);
            }
            self.partial_tab_cols = 0;
            self.current_col = 0;
            self.parse_html_block_line(events);
        }
    }

'''
SCAN = '''            let kind = self.html_block.unwrap();
            let start = self.cursor.offset();
            self.skip_indent();
            if matches!(kind, HtmlBlockKind::Type6 | HtmlBlockKind::Type7)
                && (self.cursor.is_eof() || self.at_line_ending())
            {
                // Leave blank-line state transitions in the existing handler.
                self.cursor = Cursor::new_at(self.input, start);
                self.current_col = 0;
                self.parse_html_block_line(events);
                continue;
            }
            let content_start = self.cursor.offset();
            let rest = self.cursor.remaining_slice();
            let distance = memchr::memchr(b'\\n', rest).unwrap_or(rest.len());
            let line_end = content_start + distance;
            let end = line_end + usize::from(line_end < self.input.len());
            self.cursor = Cursor::new_at(self.input, end);
            events.push(BlockEvent::HtmlBlockText(Range::from_usize(start, end)));
            if self.html_block_ends(kind, &self.input[content_start..line_end]) {
                self.html_block = None;
                events.push(BlockEvent::HtmlBlockEnd);
            }
'''
def variant(name,files):
 if name in ['production','dispatch-after-line']:
  from pathlib import Path
  source='production-parser.rs' if name=='production' else 'dispatch-after-line-parser.rs'
  files['src/block/parser.rs']=(Path(__file__).parent/source).read_text()
  return files
 if name=='baseline':return files
 s=files['src/block/parser.rs']
 for part in name.split('+'):
  if part in ['run','scan','iter']:
   s=s.replace('            self.parse_line(events);\n','''            self.parse_line(events);
            if self.html_block.is_some() && self.container_stack.is_empty() {
                self.parse_html_block_run(events);
            }
''',1)
   method=RUN if part=='run' else RUN.replace('            self.parse_html_block_line(events);\n',SCAN)
   if part=='iter':
    method=method.replace('        while self.html_block.is_some() && !self.cursor.is_eof() {', """        let input = self.input;
        let offset = self.cursor.offset();
        let ends = memchr::memchr_iter(b'\\n', &input[offset..])
            .map(|i| offset + i)
            .chain(std::iter::once(input.len()));
        for line_end in ends {
            if self.html_block.is_none() || self.cursor.is_eof() {
                break;
            }""")
    method=method.replace("            let rest = self.cursor.remaining_slice();\n            let distance = memchr::memchr(b'\\n', rest).unwrap_or(rest.len());\n            let line_end = content_start + distance;\n", "")
   s=s.replace('    /// Parse a single HTML block line after container matching.',method+'    /// Parse a single HTML block line after container matching.',1)
  elif part=='entry':
   s=s.replace("            if self.html_block.is_some() && self.container_stack.is_empty() {\n                self.parse_html_block_run(events);\n            }\n", "", 1)
   s=s.replace("        self.parse_html_block_line(events);\n        true\n", "        self.parse_html_block_line(events);\n        if self.html_block.is_some() && self.container_stack.is_empty() {\n            self.parse_html_block_run(events);\n        }\n        true\n", 1)
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
  elif part=='tags':
   import re,collections
   a=s.index('        const BLOCK_TAGS:');b=s.index('\n    }',a)
   tags=re.findall(r'b"([a-z]+)"',s[a:b]);assert len(tags)==56
   groups=collections.defaultdict(list)
   for tag in tags:groups[(len(tag),tag[0])].append(tag)
   body="        let Some(&first) = name.first() else { return false; };\n        match (name.len(), first.to_ascii_lowercase()) {\n"
   for (length,first),names in sorted(groups.items()):
    body+=f"            ({length}, b'{first}') => "+' || '.join(f'self.eq_ignore_ascii_case(name, b"{tag}")' for tag in names)+",\n"
   body+="            _ => false,\n        }"
   s=s[:a]+body+s[b:]
  elif part in ['markers','markers-lite']:
   a=s.index('        match kind {',s.index('    fn html_block_ends'));b=s.index('\n    /// Get the current line slice',a)
   s=s[:a]+'''        match kind {
            HtmlBlockKind::Type1 => memchr::memchr_iter(b'<', line).any(|i| {
                let tail = &line[i..];
                [b"</script".as_slice(), b"</pre", b"</style", b"</textarea"]
                    .iter()
                    .any(|needle| tail.get(..needle.len()).is_some_and(|candidate| {
                        candidate.eq_ignore_ascii_case(needle)
                    }))
            }),
            HtmlBlockKind::Type2 => memchr::memmem::find(line, b"-->").is_some(),
            HtmlBlockKind::Type3 => memchr::memmem::find(line, b"?>").is_some(),
            HtmlBlockKind::Type4 => memchr::memmem::find(line, b"]]>").is_some(),
            HtmlBlockKind::Type5 => memchr::memchr(b'>', line).is_some(),
            HtmlBlockKind::Type6 | HtmlBlockKind::Type7 => false,
        }
    }
''' +s[b:]
   if part=='markers-lite':
    for needle,byte,tail,n in [('b"-->"',"b'-'",'b"->"',3),('b"?>"',"b'?'",'b">"',2),('b"]]>"',"b']'",'b"]>"',3)]:
     s=s.replace(f'memchr::memmem::find(line, {needle}).is_some()', f'memchr::memchr_iter({byte}, line).any(|i| line.get(i + 1..i + {n}) == Some({tail}.as_slice()))')
   a=s.index('    #[inline]\n    fn contains_bytes(');b=s.index('    /// Find end of current line',a);s=s[:a]+s[b:]
  else:raise ValueError(part)
 files['src/block/parser.rs']=s
 return files
