URL SIMD Variant A initially failed its mixed Unicode boundary test.

Test: `html::autolink::tests::preserves_mixed_utf8_and_cjk_boundaries`
Input: `"https://example.test/é中/path`tail"`, `from = 0`.
Observed: candidate returned `36` (string end); scalar oracle returned `31`
(the backtick terminator).

Cause: the first draft's Unicode continuation loop omitted the original ASCII
terminator check after the wide pass stopped at the first high-bit byte. It
decoded Unicode correctly but treated the later backtick as ordinary text.

Corrective diff applied to the final URL patch:

```diff
 while end < bytes.len() {
+    let byte = bytes[end];
+    if byte.is_ascii() {
+        if !is_url_byte(byte) {
+            break;
+        }
+        end += 1;
+        continue;
+    }
     let Some(ch) = s.get(end..).and_then(|rest| rest.chars().next()) else {
```

The focused URL differential tests pass after this fix, including mixed UTF-8,
CJK/fullwidth punctuation, ASCII terminators, tails, and every valid boundary
offset.
