from pathlib import Path
W=Path(__file__).parent

def change(s,a,b):
 assert a in s,a[:100]
 return s.replace(a,b,1)

def transform(names,files):
 for name in names.split('+'):
  if name=='baseline':continue
  if name.startswith('nibble'):
   f='src/byte_search.rs';s=files[f]
   s=change(s,'    membership: [bool; 256],','''    membership: [bool; 256],
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    low: [u8;16],
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    high: [u8;16],
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    nibble: bool,''')
   s=change(s,'        Self {\n            bytes: *bytes,\n            membership,\n        }','''        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        let (low, high, nibble) = {
            let mut rows = [0u16;16];
            let mut i=0;while i<N {rows[(bytes[i]>>4) as usize] |= 1u16 << (bytes[i]&15);i+=1;}
            let mut low=[0u8;16];let mut high=[0u8;16];let mut groups=0;
            let mut h=0;let mut valid=true;
            while h<16 {
                if rows[h]!=0 {
                    let mut previous=0;while previous<h && rows[previous]!=rows[h] {previous+=1;}
                    if previous<h {high[h]=high[previous];}
                    else if groups<8 {
                        let bit=1u8 << groups;groups+=1;high[h]=bit;
                        let mut l=0;while l<16 {if rows[h]&(1<<l)!=0 {low[l]|=bit;}l+=1;}
                    } else {valid=false;}
                }
                h+=1;
            }
            (low,high,valid)
        };
        Self { bytes:*bytes, membership,
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] low,
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] high,
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] nibble,
        }''')
   marker='''            let mut mask = vdupq_n_u8(0);
            for &byte in &self.bytes {
                mask = vorrq_u8(mask, vceqq_u8(v, vdupq_n_u8(byte)));
            }'''
   repl='''            let mask = if self.nibble {
                let lo=vqtbl1q_u8(vld1q_u8(self.low.as_ptr()),vandq_u8(v,vdupq_n_u8(15)));
                let hi=vqtbl1q_u8(vld1q_u8(self.high.as_ptr()),vshrq_n_u8::<4>(v));
                vtstq_u8(lo,hi)
            } else {
                let mut mask=vdupq_n_u8(0);
                for &byte in &self.bytes {mask=vorrq_u8(mask,vceqq_u8(v,vdupq_n_u8(byte)));}
                mask
            };'''
   s=change(s,marker,repl)
   if name=='nibble-locate':
    start=s.index('    fn first_in_chunk(&self, input: &[u8]) -> Option<usize> {',s.index('fn chunk_has_match(&self, input: &[u8]) -> bool {',s.index('target_feature = "neon"))]\n    #[inline]\n    fn chunk_has_match')))
    s=s[:start]+s[start:].replace('''        self.find_scalar(input)''','''        let mask=self.neon_mask(input);
        (mask!=0).then(|| (mask.trailing_zeros()/4) as usize)''',1)
    # Locate once, replacing the boolean probe + scalar rescan for ARM.
    s=change(s,'            while len - pos >= 16 {','''            while len - pos >= 16 {
                #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
                {
                    if let Some(i)=self.first_in_chunk(&input[pos..pos+16]) {return Some(pos+i);}
                    pos+=16;continue;
                }
                #[cfg(target_arch = "x86_64")]
                {''')
    s=change(s,'                pos += 16;\n            }','                pos += 16;\n                }\n            }')
    # Helper duplicates classification for exact lane extraction.
    helper='''
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    #[inline]
    fn neon_mask(&self,input:&[u8])->u64 {
        use core::arch::aarch64::*;
        assert!(input.len()>=16);
        // SAFETY: a full readable vector is required; table arrays contain 16 bytes.
        unsafe {
            let v=vld1q_u8(input.as_ptr());
'''+repl+'''
            vget_lane_u64(vreinterpret_u64_u8(vshrn_n_u16::<4>(vreinterpretq_u16_u8(mask))),0)
        }
    }
'''
    i=s.rfind('\n}');s=s[:i]+helper+s[i:]
   files[f]=s
  elif name=='direct-text-event':
   f='src/lib.rs';s=files[f];marker='    let mut image_state = None;\n'
   s=change(s,marker,"""    if let [InlineEvent::Text(range)]=inline_events.as_slice() {writer.write_text_with_entities(range.slice(text));return;}
"""+marker);files[f]=s
  elif name=='code-info-borrow':
   f='src/render.rs';s=files[f]
   s=change(s,'fn unescape_backslashes(input: &[u8]) -> Vec<u8> {', "fn unescape_backslashes(input: &[u8]) -> std::borrow::Cow<'_, [u8]> {\n        if memchr::memchr(b'\\\\',input).is_none() {return std::borrow::Cow::Borrowed(input);}")
   a=s.index('fn unescape_backslashes(');b=s.index('\n    }',a);part=s[a:b];part=part.rsplit('        out',1)
   if len(part)!=2:
    # The existing local output name is inspected below when applying.
    raise ValueError('missing result variable')
   s=s[:a]+part[0]+'        std::borrow::Cow::Owned(out)'+part[1]+s[b:];files[f]=s
  elif name=='heading-cache':
   f='src/lib.rs';s=files[f]
   s=change(s,'    slug_buf: Vec<u8>,','    slug_buf: Vec<u8>,\n    slug_cache: std::collections::HashMap<Box<[u8]>,Vec<u8>,rustc_hash::FxBuildHasher>,')
   s=change(s,'            slug_buf: Vec::with_capacity(64),','            slug_buf: Vec::with_capacity(64),\n            slug_cache: Default::default(),')
   s=change(s,'        self.arena.clear();','        self.arena.clear();\n        self.slug_cache.clear();')
   s=change(s,'        generate_slug_into(raw, &mut self.slug_buf);',"""        if let Some(slug)=self.slug_cache.get(raw) {self.slug_buf.clear();self.slug_buf.extend_from_slice(slug);} else {generate_slug_into(raw,&mut self.slug_buf);self.slug_cache.insert(raw.to_vec().into_boxed_slice(),self.slug_buf.clone());}
""");files[f]=s
  elif name=='first-marker':
   f='src/inline/mod.rs';s=files[f]
   a=s.index('        let has_specials = if highlight && caret_syntax');b=s.index('\n\n',a)
   s=s[:a]+"""        let first_special = if highlight && caret_syntax {
            simd::next_mark_special::<true,true>(text)
        } else if highlight {simd::next_mark_special::<true,false>(text)}
        else if caret_syntax {simd::next_mark_special::<false,true>(text)}
        else {simd::next_mark_special::<false,false>(text)};
        let has_specials=first_special.is_some();"""+s[b:]
   for fun,args in [('collect_marks_highlight_superscript','true,true'),('collect_marks_highlight','true,false'),('collect_marks_superscript','false,true'),('collect_marks','false,false')]:
    s=s.replace(fun+'(text, &mut self.mark_buffer)',f'marks::collect_marks_from::<{args}>(text, &mut self.mark_buffer,first_special.unwrap())')
    s=s.replace(fun+',','')
   # Keep profiling-only helper wrappers for existing unit tests.
   s=s.replace('fn has_inline_specials', '#[allow(dead_code)]\nfn has_inline_specials')
   files[f]=s;f='src/inline/marks.rs';s=files[f]
   for a in ['false, false','true, false','false, true','true, true']:
    s=s.replace(f'collect_marks_impl::<{a}>(text, buffer)',f'collect_marks_impl::<{a}>(text, buffer,0)')
   a=s.index('fn collect_marks_impl<');b=s.index('    let len = text.len();',a)
   part=s[a:b].replace('    buffer: &mut MarkBuffer,','    buffer: &mut MarkBuffer,\n    from:usize,').replace('let mut pos = 0;','let mut pos = from;')
   s=s[:a]+part+s[b:]
   s+='\npub(crate) fn collect_marks_from<const H:bool,const S:bool>(text:&[u8],buffer:&mut MarkBuffer,from:usize)->MarkSummary {assert_input_size(text.len());collect_marks_impl::<H,S>(text,buffer,from)}\n'
   files[f]=s
  elif name=='headings-count':
   f='src/block/parser.rs';s=files[f]
   s=change(s,"pub struct BlockParser<'a> {", "pub struct BlockParser<'a> {\n    pub(crate) heading_count: usize,")
   s=change(s,'            cursor: Cursor::new(input),','            cursor: Cursor::new(input),\n            heading_count: 0,')
   s=s.replace('        events.push(BlockEvent::HeadingStart { level });','        self.heading_count+=1;\n        events.push(BlockEvent::HeadingStart { level });');files[f]=s
   f='src/lib.rs';s=files[f];a=s.index('fn render_to_writer_with_state<');part=s[a:]
   part=change(part,'    parser.parse(events);','    parser.parse(events);\n    let heading_count=parser.heading_count;')
   part=change(part,'    render_state.reset(options);',"""    render_state.reset(options);
    if options.heading_ids && heading_count>0 {let tracker=render_state.heading_id_tracker.get_or_insert_with(HeadingIdTracker::new);tracker.used.reserve(heading_count);tracker.arena.reserve(heading_count.saturating_mul(16));}
""");s=s[:a]+part;files[f]=s
  elif name=='headings-presize':
   f='src/lib.rs';s=files[f];marker='    fn render_events(&mut self, input: &[u8], mut events: &[BlockEvent]) {\n'
   s=change(s,marker,marker+"""        if self.options.heading_ids {
            let count=events.iter().filter(|e|matches!(e,BlockEvent::HeadingStart{..})).count();
            if count>0 {let tracker=self.heading_id_tracker.get_or_insert_with(HeadingIdTracker::new);tracker.used.reserve(count);tracker.arena.reserve(count.saturating_mul(16));}
        }
""");files[f]=s
  elif name=='output2x':
   f='src/render.rs';files[f]=change(files[f],'(input_len + input_len / 4).max(64)','input_len.saturating_mul(2).max(64)')
  elif name=='escape-swar':
   f='src/escape.rs';s=files[f];start=s.index('fn first_escape_short<const ATTR: bool>');end=s.index('\n}\n',start)+2
   s=s[:start]+'''fn first_escape_short<const ATTR: bool>(input: &[u8]) -> Option<usize> {
    const ONES:u64=0x0101010101010101;
    const HIGH:u64=0x8080808080808080;
    fn zero(x:u64)->u64 {x.wrapping_sub(ONES)&!x&HIGH}
    let mut i=0;
    while input.len()-i>=8 {
        let x=u64::from_le_bytes(input[i..i+8].try_into().unwrap());
        let mut m=zero((x|(ONES*2))^(ONES*62))|zero(x^(ONES*38))|zero(x^(ONES*34));
        if ATTR {m|=zero(x^(ONES*39));}
        if m!=0 {return Some(i+(m.trailing_zeros()/8) as usize);}
        i+=8;
    }
    input[i..].iter().position(|&b| matches!(b,b'<'|b'>'|b'&'|b'"') || (ATTR && b==b'\\'')).map(|n|n+i)
}'''+s[end:]
   # ByteSet constants are unused in this isolated alternative.
   s=s.replace('const TEXT_SPECIALS:', '#[allow(dead_code)]\nconst TEXT_SPECIALS:').replace('const ATTR_SPECIALS:', '#[allow(dead_code)]\nconst ATTR_SPECIALS:');files[f]=s
  elif name=='headings-flat':
   f='src/lib.rs';s=files[f]
   s=change(s,'std::collections::HashMap<u64, SmallVec<[SlugEntry; 1]>, rustc_hash::FxBuildHasher>', 'std::collections::HashMap<u64, usize, rustc_hash::FxBuildHasher>')
   s=change(s,'    slug_buf: Vec<u8>,','    slug_buf: Vec<u8>,\n    entries: Vec<SlugEntry>,')
   s=change(s,'    next_suffix: usize,\n}', '    next_suffix: usize,\n    next: Option<usize>,\n}')
   s=change(s,'            slug_buf: Vec::with_capacity(64),','            slug_buf: Vec::with_capacity(64),\n            entries: Vec::new(),')
   s=change(s,'        self.arena.clear();','        self.arena.clear();\n        self.entries.clear();')
   start=s.index('    fn find_entry(&self, hash: u64');end=s.index('\n}\n\nfn hash_bytes',start)
   s=s[:start]+'''    fn find_entry(&self, hash:u64,slug:&[u8])->Option<&SlugEntry> {
        let mut index=self.used.get(&hash).copied();
        while let Some(i)=index {let entry=&self.entries[i];if &self.arena[entry.start..entry.start+entry.len]==slug {return Some(entry);}index=entry.next;}
        None
    }
    fn record_id(&mut self,hash:u64) {
        let start=self.arena.len();let len=self.slug_buf.len();self.arena.extend_from_slice(&self.slug_buf);
        let index=self.entries.len();let next=self.used.insert(hash,index);
        self.entries.push(SlugEntry{start,len,next_suffix:1,next});
    }
    fn set_next_suffix(&mut self,hash:u64,base_start:usize,base_len:usize,next_suffix:usize) {
        let mut index=self.used.get(&hash).copied();
        while let Some(i)=index {let entry=&mut self.entries[i];if entry.start==base_start && entry.len==base_len {entry.next_suffix=next_suffix;return;}index=entry.next;}
    }
'''+s[end:];files[f]=s
  elif name=='borrow-contiguous':
   files=transform('borrow-content',files)
   f='src/lib.rs';s=files[f]
   s=s.replace('        self.materialize(input);self.content.extend_from_slice(range.slice(input));','        if let Some(previous)=self.borrowed.as_mut() {if previous.end==range.start {previous.end=range.end;return;}}\n        self.materialize(input);self.content.extend_from_slice(range.slice(input));')
   s=s.replace('fn add_soft_break(&mut self,input:&[u8]) {\n        self.materialize(input);',"fn add_soft_break(&mut self,input:&[u8]) {\n        if let Some(range)=self.borrowed.as_mut() {if input.get(range.end as usize)==Some(&b'\\n') {range.end+=1;return;}}\n        self.materialize(input);")
   files[f]=s
  elif name=='borrow-content':
   f='src/lib.rs';s=files[f]
   # One range per document-state accumulator; materialize only on a second fragment.
   for struct,nextstruct in [('ParagraphState','HeadingState'),('HeadingState','HeadingIdTracker')]:
    start=s.index('struct '+struct+' {');end=s.index('struct '+nextstruct+' {',start)
    part=s[start:end];part=part.replace('    content: Vec<u8>,','    content: Vec<u8>,\n    borrowed: Option<Range>,')
    part=part.replace('            content: Vec::new(),','            content: Vec::new(),\n            borrowed: None,')
    part=part.replace('        self.content.clear();','        self.content.clear();\n        self.borrowed=None;')
    a=part.index('    fn add_text(');b=part.index('    fn add_soft_break(',a)
    part=part[:a]+'''    fn add_text(&mut self,input:&[u8],range:Range) {
        if self.borrowed.is_none() && self.content.is_empty() {self.borrowed=Some(range);return;}
        self.materialize(input);self.content.extend_from_slice(range.slice(input));
    }
    fn materialize(&mut self,input:&[u8]) {if let Some(range)=self.borrowed.take(){self.content.extend_from_slice(range.slice(input));}}
'''+part[b:]
    part=part.replace('fn add_soft_break(&mut self) {','fn add_soft_break(&mut self,input:&[u8]) {\n        self.materialize(input);')
    part=part.replace('fn finish(&mut self) -> &[u8] {',"fn finish<'a>(&'a mut self,input:&'a [u8]) -> &'a [u8] {")
    # Keep both paths' trailing whitespace contract.
    token='        // CommonMark: strip trailing spaces/tabs from paragraph content' if struct=='ParagraphState' else '        while self'
    insert='''        if let Some(range)=self.borrowed {let text=range.slice(input);let end=text.iter().rposition(|&b| !matches!(b,b' '|b'\\t')).map_or(0,|i|i+1);return &text[..end];}
'''
    part=part.replace(token,insert+token,1)
    s=s[:start]+part+s[end:]
   s=s.replace('para_state.finish()','para_state.finish(input)').replace('heading_state.finish()','heading_state.finish(input)')
   s=s.replace('para_state.add_text(text)','para_state.add_text(input, *range)').replace('heading_state.add_text(text)','heading_state.add_text(input, *range)')
   s=s.replace('para_state.add_soft_break()','para_state.add_soft_break(input)').replace('heading_state.add_soft_break()','heading_state.add_soft_break(input)')
   files[f]=s
  else:raise ValueError(name)
 return files
