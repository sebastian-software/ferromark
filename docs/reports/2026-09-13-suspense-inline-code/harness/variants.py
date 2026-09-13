def variant(name, files):
    if name == 'outline-probe':
        files = variant('linear-formatted', files)
        p='src/inline/mod.rs';s=files[p]
        a=s.index('        // Filter autolinks that start inside code spans')
        b=s.index('        // Fourth: collect bracket positions',a)
        block=s[a:b]
        block=block[block.index('        // Both sequences'):]
        block=block.replace('self.autolinks','autolinks').replace('self.code_spans','code_spans')
        block='\n'.join(line[4:] if line.startswith('    ') else line for line in block.split('\n'))
        s=s[:a]+'        filter_autolinks_in_code_spans(&mut self.autolinks, &self.code_spans);\n\n'+s[b:]
        i=s.index('fn filter_html_spans_in_code_spans(')
        s=s[:i]+'#[inline(never)]\nfn filter_autolinks_in_code_spans(autolinks: &mut Vec<Autolink>, code_spans: &[CodeSpan]) {\n'+block+'}\n\n'+s[i:]
        files[p]=s
        return files
    if name == 'skip-empty-code-filters':
        files = variant('tag-no-scratch', files)
        p = 'src/inline/mod.rs';s = files[p]
        a = '        filter_html_spans_in_code_spans(&mut self.html_spans, &self.code_spans);'
        assert s.count(a) == 1
        s = s.replace(a, '        if !self.code_spans.is_empty() {\n    '+a+'\n        }')
        start = s.index('        // Filter autolinks that start inside code spans')
        end = s.index('        // Fourth: collect bracket positions',start)
        block = s[start:end]
        s = s[:start]+'        if !self.code_spans.is_empty() {\n'+block+'        }\n\n'+s[end:]
        files[p]=s
        return files
    if name == 'tag-no-scratch':
        files = variant('production', files)
        p = 'src/inline/mod.rs'; s = files[p]
        s = s.replace('try_emit_tag_code_prose(text, self.mark_buffer.marks(), &mut self.code_spans, events)', 'try_emit_tag_code_prose(text, self.mark_buffer.marks(), events)')
        start = s.index('fn try_emit_tag_code_prose(')
        end = s.index('#[inline]\nfn has_inline_specials', start)
        b = '''fn try_emit_tag_code_prose(text: &[u8], marks: &[Mark], events: &mut Vec<InlineEvent>) -> bool {
    let mut i = 0;
    let mut count = 0;
    while i < marks.len() {
        let opener = marks[i];
        if opener.ch != b'`' { return false; }
        let mut j = i + 1;
        while j < marks.len() && marks[j].ch == b'<' { j += 1; }
        if j == marks.len() || marks[j].ch != b'`' || marks[j].len() != opener.len() { return false; }
        for mark in &marks[i+1..j] {
            if parse_inline_html(text, mark.pos as usize).is_some_and(|end| end > marks[j].pos as usize) { return false; }
        }
        count += 1;
        i = j + 1;
    }
    events.reserve(count*2+1);
    let mut pos = 0;
    i = 0;
    while i < marks.len() {
        let opener = marks[i];
        let mut j = i + 1;
        while marks[j].ch == b'<' { j += 1; }
        let closer = marks[j];
        if opener.pos > pos { events.push(InlineEvent::Text(Range::from_usize(pos as usize, opener.pos as usize))); }
        events.push(InlineEvent::Code(trim_span_padding(text, opener.end, closer.pos)));
        pos = closer.end;
        i = j + 1;
    }
    if (pos as usize) < text.len() { events.push(InlineEvent::Text(Range::from_usize(pos as usize, text.len()))); }
    true
}

'''
        files[p] = s[:start]+b+s[end:]
        return files
    if name == 'late-tag-prose':
        files = variant('production', files)
        p = 'src/inline/mod.rs'; s = files[p]
        start = s.index('        // Tag names in inline code otherwise')
        end = s.index('        // Phase 2: Resolve marks by precedence', start)
        block = s[start:end]
        s = s[:start]+s[end:]
        block = block.replace('if summary.has_code()\n            && summary.has_less_than()', 'if summary.has_less_than()')
        block = '\n'.join('    '+line if line else '' for line in block.split('\n'))
        marker = '        if summary.has_code() {\n            resolve_code_spans('
        assert s.count(marker) == 1
        files[p] = s.replace(marker, '        if summary.has_code() {\n'+block+'            resolve_code_spans(',1)
        return files
    if name in ['production','linear-formatted','final']:
        from pathlib import Path
        for f in ['src/inline/mod.rs', 'src/inline/code_span.rs']:
            files[f] = (Path(__file__).parent/(name+'-source')/f).read_text()
        return files
    if name in ['baseline','baseline-copy']:
        return files
    if name == 'linear-autolink':
        return variant('autolink-code', variant('linear-ranges', files))
    if name == 'linear-tag-prose':
        files = variant('linear-prose', files)
        files['src/inline/mod.rs'] = files['src/inline/mod.rs'].replace('if summary.has_code() && !may_have_autolinks', 'if summary.has_code() && summary.has_less_than() && !may_have_autolinks')
        return files
    if name in ['code-prose-outlined', 'linear-prose']:
        if name == 'linear-prose':
            files = variant('linear-ranges', files)
        files = variant('code-prose', files)
        files['src/inline/mod.rs'] = files['src/inline/mod.rs'].replace('fn try_emit_code_prose(', '#[inline(never)]\nfn try_emit_code_prose(')
        return files
    if name in ['autolink-fused', 'combo']:
        p = 'src/inline/links.rs'
        s = files[p]
        a = "    while pos < len && !matches!(text[pos], 0..=b' ' | b'<' | b'>' | 0x7f) {\n        pos += 1;\n    }"
        b = "    let mut possible = false;\n    while pos < len && !matches!(text[pos], 0..=b' ' | b'<' | b'>' | 0x7f) {\n        possible |= matches!(text[pos], b':' | b'@');\n        pos += 1;\n    }\n    if !possible { return None; }"
        assert s.count(a) == 1
        files[p] = s.replace(a, b)
    if name in ['autolink-code', 'combo']:
        p = 'src/inline/mod.rs'
        s = files[p]
        s = s.replace('find_autolink_literals_into, find_autolinks_into, resolve_links_into_with_limits,', 'find_autolink_literals_into, find_autolinks_outside_code, resolve_links_into_with_limits,')
        a = '        if has_lt {\n            find_autolinks_into(text, &mut self.autolinks);\n        }\n'
        assert s.count(a) == 1
        s = s.replace(a, '')
        a = '        // Filter autolinks that start inside code spans\n'
        assert s.count(a) == 1
        s = s.replace(a, '        if has_lt {\n            find_autolinks_outside_code(text, &self.code_spans, &mut self.autolinks);\n        }\n\n' + a)
        files[p] = s
        p = 'src/inline/links.rs'
        s = files[p]
        a = '/// Try to parse an autolink at the given position.\n'
        b = '''pub(super) fn find_autolinks_outside_code(text: &[u8], code: &[super::code_span::CodeSpan], out: &mut Vec<Autolink>) {
    out.clear();
    let mut pos = 0;
    let mut ci = 0;
    while let Some(offset) = memchr::memchr(b'<' , &text[pos..]) {
        let idx = pos + offset;
        while ci < code.len() && idx >= code[ci].closer_end as usize { ci += 1; }
        if ci < code.len() && idx >= code[ci].opener_pos as usize {
            pos = code[ci].closer_end as usize;
            continue;
        }
        if let Some(autolink) = try_parse_autolink(text, idx) {
            pos = autolink.end as usize;
            out.push(autolink);
        } else { pos = idx + 1; }
    }
}

'''
        assert s.count(a) == 1
        files[p] = s.replace(a, b+a)
    if name in ['code-emit', 'combo']:
        p = 'src/inline/mod.rs'
        s = files[p]
        a = '        // Build sorted list of events to emit\n'
        b = '''        if !code_spans.is_empty()
            && math_spans.is_empty() && emphasis_matches.is_empty()
            && strikethrough_matches.is_empty() && subscript_matches.is_empty()
            && superscript_matches.is_empty() && highlight_matches.is_empty()
            && autolink_literals.is_empty() && resolved_links.is_empty()
            && resolved_ref_links.is_empty() && autolinks.is_empty()
            && html_spans.is_empty() && footnote_refs.is_empty() && inline_footnotes.is_empty()
            && marks.iter().all(|m| m.flags & flags::IN_CODE != 0 || !matches!(m.ch, b'\\\\' | b'\\n'))
        {
            for span in code_spans {
                if span.opener_pos > pos {
                    events.push(InlineEvent::Text(Range::from_usize(pos as usize, span.opener_pos as usize)));
                }
                events.push(InlineEvent::Code(trim_span_padding(text, span.opener_end, span.closer_pos)));
                pos = span.closer_end;
            }
            if pos < text_len { events.push(InlineEvent::Text(Range::from_usize(pos as usize, text_len as usize))); }
            return;
        }

'''
        assert s.count(a) == 1
        files[p] = s.replace(a, b+a)
    if name == 'code-prose':
        p = 'src/inline/mod.rs'
        s = files[p]
        a = '        // Phase 2: Resolve marks by precedence\n'
        b = '''        if summary.has_code() && !may_have_autolinks
            && !self.mark_buffer.limit_exceeded()
            && !self.mark_buffer.code_span_limit_exceeded()
            && try_emit_code_prose(text, self.mark_buffer.marks(), &mut self.code_spans, events)
        {
            return;
        }

'''
        assert s.count(a) == 1
        s = s.replace(a, b+a)
        a = '#[inline]\nfn has_inline_specials(input: &[u8]) -> bool {'
        b = '''fn try_emit_code_prose(text: &[u8], marks: &[Mark], spans: &mut Vec<CodeSpan>, events: &mut Vec<InlineEvent>) -> bool {
    spans.clear();
    let mut i = 0;
    while i < marks.len() {
        let opener = marks[i];
        if opener.ch != b'`' { return false; }
        let mut j = i + 1;
        while j < marks.len() && marks[j].ch == b'<' { j += 1; }
        if j == marks.len() || marks[j].ch != b'`' || marks[j].len() != opener.len() { return false; }
        let closer = marks[j];
        for mark in &marks[i+1..j] {
            if parse_inline_html(text, mark.pos as usize).is_some_and(|end| end > closer.pos as usize) { return false; }
        }
        spans.push(CodeSpan { opener_pos: opener.pos, opener_end: opener.end, closer_pos: closer.pos, closer_end: closer.end });
        i = j + 1;
    }
    events.reserve(spans.len()*2+1);
    let mut pos = 0;
    for span in spans {
        if span.opener_pos > pos { events.push(InlineEvent::Text(Range::from_usize(pos as usize, span.opener_pos as usize))); }
        events.push(InlineEvent::Code(trim_span_padding(text, span.opener_end, span.closer_pos)));
        pos = span.closer_end;
    }
    if (pos as usize) < text.len() { events.push(InlineEvent::Text(Range::from_usize(pos as usize, text.len()))); }
    true
}

'''
        assert s.count(a) == 1
        files[p] = s.replace(a,b+a)
    if name == 'linear-ranges':
        p = 'src/inline/code_span.rs'
        s = files[p]
        s = s.replace('    let len = marks.len();\n', '    let len = marks.len();\n    let mut html_idx = 0;\n',1)
        s = s.replace('        if pos_in_spans(opener_pos as u32, html_spans) {', '''        while html_idx < html_spans.len() && opener_pos as u32 >= html_spans[html_idx].1 { html_idx += 1; }
        if html_idx < html_spans.len() && opener_pos as u32 >= html_spans[html_idx].0 {''')
        s = s.replace('''#[inline]
fn pos_in_spans(pos: u32, spans: &[(u32, u32)]) -> bool {
    spans.iter().any(|&(start, end)| pos >= start && pos < end)
}

''','')
        files[p] = s
        p = 'src/inline/mod.rs'
        s = files[p]
        a = '''        self.autolinks.retain(|al| {
            !self
                .code_spans
                .iter()
                .any(|cs| al.start >= cs.opener_pos && al.start < cs.closer_end)
        });'''
        b = '''        let mut code_idx = 0;
        self.autolinks.retain(|al| {
            while code_idx < self.code_spans.len() && al.start >= self.code_spans[code_idx].closer_end { code_idx += 1; }
            code_idx == self.code_spans.len() || al.start < self.code_spans[code_idx].opener_pos
        });'''
        assert s.count(a) == 1
        s = s.replace(a,b)
        a = '''    spans.retain(|span| {
        !code_spans
            .iter()
            .any(|cs| span.start >= cs.opener_pos && span.start < cs.closer_end)
    });'''
        b = '''    let mut code_idx = 0;
    spans.retain(|span| {
        while code_idx < code_spans.len() && span.start >= code_spans[code_idx].closer_end { code_idx += 1; }
        code_idx == code_spans.len() || span.start < code_spans[code_idx].opener_pos
    });'''
        assert s.count(a) == 1
        files[p] = s.replace(a,b)
    assert name in ['autolink-fused', 'autolink-code', 'code-emit', 'combo', 'code-prose', 'linear-ranges'], name
    return files
