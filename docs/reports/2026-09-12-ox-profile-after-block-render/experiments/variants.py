def change(files,path,old,new):
 assert files[path].count(old)==1,(path,old[:90],files[path].count(old))
 files[path]=files[path].replace(old,new)
def variant(name,files):
 if name=='baseline':return files
 if name=='autolink-prefilter':
  change(files,'src/inline/links.rs',"    // Check if it's a valid URL or email", '''    // A valid URI needs a colon; a valid email needs an at sign.
    if memchr::memchr2(b':', b'@', content).is_none() { return None; }

    // Check if it's a valid URL or email''')
  return files
 if name=='softbreak-guarded':
  files=variant('softbreak-no-code',files)
  change(files,'src/inline/mod.rs','coalesce_softbreaks && code_spans.is_empty()\n', 'coalesce_softbreaks && code_spans.is_empty()\n            && memchr::memchr2(0, b"\\r"[0], text).is_none()\n')
  return files
 if name=='softbreak-no-code':
  files=variant('softbreak-ranges',files)
  change(files,'src/inline/mod.rs','let coalesce_softbreaks = coalesce_softbreaks\n', 'let coalesce_softbreaks = coalesce_softbreaks && code_spans.is_empty()\n')
  return files
 if name=='softbreak-ranges':
  p='src/inline/mod.rs'
  change(files,p,'pub struct InlineParser {','pub struct InlineParser {\n    pub(crate) coalesce_softbreaks: bool,')
  change(files,p,'            mark_buffer: MarkBuffer::new(),','            coalesce_softbreaks: false,\n            mark_buffer: MarkBuffer::new(),')
  change(files,p,'            &mut self.emit_suppress_ranges,\n            events,','            &mut self.emit_suppress_ranges,\n            events,\n            self.coalesce_softbreaks,')
  change(files,p,'        suppress_ranges: &mut Vec<(u32, u32)>,\n        events: &mut Vec<InlineEvent>,','        suppress_ranges: &mut Vec<(u32, u32)>,\n        events: &mut Vec<InlineEvent>,\n        coalesce_softbreaks: bool,')
  change(files,p,'        // Build sorted list of events to emit','        let coalesce_softbreaks = coalesce_softbreaks\n            && !resolved_links.iter().any(|link| link.is_image)\n            && !resolved_ref_links.iter().any(|link| link.is_image);\n\n        // Build sorted list of events to emit')
  change(files,p,'                // Soft break (newline without 2+ spaces) - also not in code or link destinations','''                // Diagnostic private-render experiment: preserve literal LF in text.
                let at = mark.pos as usize;
                if coalesce_softbreaks && mark.end == mark.pos + 1 && at > 0
                    && at + 1 < text.len()
                    && !matches!(text[at - 1], b' ' | b'\\t' | b'\\r' | b'\\n')
                    && !matches!(text[at + 1], b' ' | b'\\t' | b'\\r' | b'\\n')
                { continue; }
                // Soft break (newline without 2+ spaces) - also not in code or link destinations''')
  change(files,'src/lib.rs','    inline_parser.parse_with_options_in_document(\n        text,','    inline_parser.coalesce_softbreaks = true;\n    inline_parser.parse_with_options_in_document(\n        text,')
  change(files,'src/lib.rs','    #[cfg(feature = "profiling")]\n    profiling::record_inline_events','    inline_parser.coalesce_softbreaks = false;\n    #[cfg(feature = "profiling")]\n    profiling::record_inline_events')
 elif name=='html-blank-run':
  p='src/block/parser.rs'
  change(files,p,'            if self.coalesce_html {\n                self.parse_html_block_run::<true>(events);','''            if self.coalesce_html && !self.options.definition_lists
                && matches!(self.html_block, Some(HtmlBlockKind::Type6 | HtmlBlockKind::Type7))
            {
                self.parse_html_blank_run(events);
            } else if self.coalesce_html {
                self.parse_html_block_run::<true>(events);''')
  change(files,p,'    /// Consume root HTML continuations without repeating general block dispatch.','''    // Diagnostic: defer root type-6/7 state updates until the stopping line.
    fn parse_html_blank_run(&mut self, events: &mut Vec<BlockEvent>) {
        let start = self.cursor.offset();
        let mut at = start;
        let mut last_start = start;
        while at < self.input.len() {
            last_start = at;
            let mut content = at;
            while matches!(self.input.get(content), Some(b' ' | b'\\t')) { content += 1; }
            let blank = content == self.input.len()
                || self.input[content] == b'\\n'
                || (self.input[content] == b'\\r' && self.input.get(content + 1) == Some(&b'\\n'));
            if blank {
                if at > start { events.push(BlockEvent::HtmlBlockText(Range::from_usize(start, at))); }
                self.cursor = Cursor::new_at(self.input, at);
                self.current_line_start = at;
                self.current_col = 0;
                self.partial_tab_cols = 0;
                self.parse_html_block_line(events);
                return;
            }
            let end = content + Self::find_html_line_end(&self.input[content..]);
            at = end + usize::from(end < self.input.len());
        }
        if at > start {
            self.current_line_start = last_start;
            self.current_col = 0;
            self.partial_tab_cols = 0;
            self.cursor = Cursor::new_at(self.input, last_start);
            self.skip_indent();
            self.cursor = Cursor::new_at(self.input, at);
            events.push(BlockEvent::HtmlBlockText(Range::from_usize(start, at)));
        }
    }

    /// Consume root HTML continuations without repeating general block dispatch.''')
 elif name=='escape-short-copy':
  p='src/escape.rs'
  # Restrict the experiment to text escaping; attribute/URL semantics stay put.
  marker='/// Escape HTML text content, checking for quotes as well (for attribute context).'
  a,b=files[p].split(marker,1)
  a=a.replace('out.extend_from_slice(&input[start..pos]);','push_short_text_run(out, &input[start..pos]);').replace('out.extend_from_slice(&input[start..]);','push_short_text_run(out, &input[start..]);')
  files[p]=a+'''// Diagnostic short-copy prototype, independently written for a byte Vec.
#[inline]
fn push_short_text_run(out: &mut Vec<u8>, input: &[u8]) {
    let len = input.len();
    if len > 16 { out.extend_from_slice(input); return; }
    out.reserve(len);
    let old = out.len();
    // SAFETY: reserve provides len writable bytes. Every write stays in that
    // range, all len bytes are initialized, and the exclusive output borrow
    // prevents source aliasing. The source reads stay inside input.
    unsafe {
        let dst = out.as_mut_ptr().add(old);
        let src = input.as_ptr();
        if len >= 8 {
            core::ptr::copy_nonoverlapping(src, dst, 8);
            core::ptr::copy_nonoverlapping(src.add(len - 8), dst.add(len - 8), 8);
        } else if len >= 4 {
            core::ptr::copy_nonoverlapping(src, dst, 4);
            core::ptr::copy_nonoverlapping(src.add(len - 4), dst.add(len - 4), 4);
        } else if len > 0 {
            *dst = *src;
            *dst.add(len / 2) = *src.add(len / 2);
            *dst.add(len - 1) = *src.add(len - 1);
        }
        out.set_len(old + len);
    }
}

'''+marker+b
 elif name=='escape-single-scan':
  p='src/escape.rs'
  start=files[p].index('    if input.len() <= SHORT_SCAN_MAX {',files[p].index('pub(crate) fn first_text_escape'))
  end=files[p].index('\n}',start)
  files[p]=files[p][:start]+'    TEXT_SPECIALS.find(input)'+files[p][end:]
 else:raise ValueError(name)
 return files
