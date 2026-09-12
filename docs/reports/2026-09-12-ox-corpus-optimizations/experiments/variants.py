from pathlib import Path
import re,json
W=Path(__file__).parent

def variant(name,files):
 f=files.copy()
 if name=='baseline':return f
 if name=='production':return json.loads((W/'production-source.json').read_text())
 if name=='lines':
  p='src/block/parser.rs';s=f[p]
  s=s.replace("        let end = slice\n            .iter()\n            .position(|&b| b == b'\\n')\n            .unwrap_or(slice.len());", "        let end = memchr::memchr(b'\\n', slice).unwrap_or(slice.len());")
  s=s.replace("        while !self.cursor.is_eof() && !self.cursor.at(b'\\n') {\n            parser_cursor_bump!(self.cursor);\n        }\n        self.cursor.offset()", "        let remaining = self.cursor.remaining_slice();\n        let distance = memchr::memchr(b'\\n', remaining).unwrap_or(remaining.len());\n        parser_cursor_advance!(self.cursor, distance);\n        self.cursor.offset()")
  f[p]=s
 elif name=='lazy-sort':
  p='src/inline/mod.rs';s=f[p];start=s.index('        emit_points.sort_unstable_by_key(|p| {');end=s.index('\n        });',start)+len('\n        });')
  end_kinds=' | '.join('EmitKind::'+v for v in ['CodeSpanEnd','StrongEnd','EmphasisEnd','StrikethroughEnd','SubscriptEnd','SuperscriptEnd','HighlightEnd','LinkEnd','ImageEnd'])
  s=s[:start]+f'''        emit_points.sort_unstable_by(|a, b| {{
            a.pos.cmp(&b.pos).then_with(|| {{
                let a_end = matches!(a.kind, {end_kinds});
                let b_end = matches!(b.kind, {end_kinds});
                a_end.cmp(&b_end)
            }})
        }});'''+s[end:];f[p]=s
 elif name=='ranges-after-cells':
  f=variant('all-ranges',f);p='src/lib.rs';s=f[p];start=s.index('            if let [BlockEvent::Code(first)');end=s.index('            if let [\n                BlockEvent::TableCellStart',start);body=s[start:end];s=s[:start]+s[end:];s=s.replace('            self.render_block_event(input, event);',body+'            self.render_block_event(input, event);');f[p]=s
 elif name=='packed-sort':
  p='src/inline/mod.rs';s=f[p];start=s.index('        emit_points.sort_unstable_by_key(|p| {');end=s.index('\n        });',start)+len('\n        });');body=s[start:end];match=body[body.index('matches!('):body.index('\n                ),')+len('\n                )')];s=s[:start]+'        emit_points.sort_unstable_by_key(|p| (u64::from(p.pos) << 1) | u64::from('+match+'));'+s[end:];f[p]=s
 elif name=='ordered-code':
  p='src/inline/mod.rs';s=f[p];end='''            emit_points.push(EmitPoint {
                pos: span.closer_pos,
                kind: EmitKind::CodeSpanEnd,
                end: span.closer_end,
            });''';content='''            emit_points.push(EmitPoint {
                pos: content_start,
                kind: EmitKind::CodeContent(content_end),
                end: content_end,
            });''';assert s.count(end)==1 and s.count(content)==1;s=s.replace(end+'\n','').replace(content,content+'\n'+end);f[p]=s
 elif name in ['code-ranges','html-ranges','all-ranges']:
  p='src/lib.rs';s=f[p];parts=[]
  for tag in (['Code','HtmlBlockText'] if name=='all-ranges' else ['Code'] if name=='code-ranges' else ['HtmlBlockText']):
   guard='' if tag=='Code' else '\n                    && (self.options.render_policy == RenderPolicy::Untrusted\n                        || !self.options.disallowed_raw_html)'
   parts.append(f'''            if let [BlockEvent::{tag}(first), BlockEvent::{tag}(second), ..] = events
                && first.end == second.start{guard}
            {{
                let mut range = Range::new(first.start, second.end);
                let mut consumed = 2;
                while let Some(BlockEvent::{tag}(next)) = events.get(consumed) {{
                    if next.start != range.end {{ break; }}
                    range.end = next.end;
                    consumed += 1;
                }}
                self.render_block_event(input, &BlockEvent::{tag}(range));
                events = &events[consumed..];
                continue;
            }}''')
  marker='        while let Some(event) = events.first() {';assert s.count(marker)==1;s=s.replace(marker,marker+'\n'+'\n'.join(parts));f[p]=s
 else:
  for part in name.split('+'):f=variant(part,f)
 return f
