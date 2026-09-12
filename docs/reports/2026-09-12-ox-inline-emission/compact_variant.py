from variants import once,compact_links

def compact_all(s):
 s=compact_links(s)
 # Resolve rich payloads through the existing parser-owned records.
 s=once(s,'        for autolink in autolinks {','        for (index, autolink) in autolinks.iter().enumerate() {')
 for kind in ['AutolinkUrl','AutolinkEmail']:
  s=once(s,'''kind: EmitKind::'''+kind+''' {
                        content_start: autolink.content_start,
                        content_end: autolink.content_end,
                    },''','kind: EmitKind::'+kind+' { index: index as u32 },')
  s=once(s,'''                EmitKind::'''+kind+''' {
                    content_start,
                    content_end,
                } => {''','''                EmitKind::'''+kind+''' { index } => {
                    let autolink = &autolinks[index as usize];
                    let (content_start, content_end) = (autolink.content_start, autolink.content_end);''')
  s=once(s,'    '+kind+' {\n        content_start: u32,\n        content_end: u32,\n    },','    '+kind+' { index: u32 },')
 s=once(s,'        for inline_note in inline_footnotes {','        for (index, inline_note) in inline_footnotes.iter().enumerate() {')
 s=once(s,'''kind: EmitKind::InlineFootnote {
                    content_start: inline_note.content_start,
                    content_end: inline_note.content_end,
                },''','kind: EmitKind::InlineFootnote { index: index as u32 },')
 s=once(s,'''                EmitKind::InlineFootnote {
                    content_start,
                    content_end,
                } => {''','''                EmitKind::InlineFootnote { index } => {
                    let inline_note = &inline_footnotes[index as usize];
                    let (content_start, content_end) = (inline_note.content_start, inline_note.content_end);''')
 s=once(s,'    InlineFootnote {\n        content_start: u32,\n        content_end: u32,\n    },','    InlineFootnote { index: u32 },')
 s=once(s,'                    end: al.end,\n                    kind: al.kind,','                    kind: al.kind,')
 s=once(s,'                EmitKind::AutolinkLiteral { end, kind } => {','                EmitKind::AutolinkLiteral { kind } => {\n                    let end = point.end;')
 s=once(s,'    AutolinkLiteral {\n        end: u32,\n        kind: links::AutolinkLiteralKind,\n    },','    AutolinkLiteral { kind: links::AutolinkLiteralKind },')
 s=once(s,'        for span in math_spans {','        for (index, span) in math_spans.iter().enumerate() {')
 a=s.index('            let (content_start, content_end) = span.content_range();',s.index('// Add math span events'));b=s.index('            if span.is_display {',a)
 trim=s[a:b];s=s[:a]+s[b:]
 for kind in ['MathInline','MathDisplay']:
  s=once(s,'kind: EmitKind::'+kind+' {\n                        content_start: cs as u32,\n                        content_end: ce as u32,\n                    },','kind: EmitKind::'+kind+' { index: index as u32 },')
  a=s.index('                EmitKind::'+kind+' {\n                    content_start,');b=s.index('\n                }\n',a)
  old=s[a:b];new='                EmitKind::'+kind+' { index } => {\n                    let span = &math_spans[index as usize];\n'+trim+old[old.index('                    events.push'):].replace('content_start as usize','cs').replace('content_end as usize','ce')
  s=s[:a]+new+s[b:]
  s=once(s,'    '+kind+' {\n        content_start: u32,\n        content_end: u32,\n    },','    '+kind+' { index: u32 },')
 return s


def compact_code(s):
 s=once(s,'        for span in code_spans {','        for (index, span) in code_spans.iter().enumerate() {')
 s=once(s,'            if fuse_code {\n            let (content_start, content_end) = span.content_range();','            if fuse_code {')
 s=once(s,'kind: EmitKind::CodeSpan { content_start, content_end },','kind: EmitKind::CodeSpan { index: index as u32 },')
 s=once(s,'                EmitKind::CodeSpan { content_start, content_end } => {','                EmitKind::CodeSpan { index } => {\n                    let (content_start, content_end) = code_spans[index as usize].content_range();')
 s=once(s,'    CodeSpan { content_start: u32, content_end: u32 },','    CodeSpan { index: u32 },')
 return s
