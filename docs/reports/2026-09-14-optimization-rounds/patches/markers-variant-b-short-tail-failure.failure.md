Marker Variant B failed before the final short-tail fix.

Test: `parser::inline::marker_scan::tests::matches_scalar_oracle_for_all_option_masks_and_offsets`
Input: option mask `1` (MDX only), `len = 1`, `from = 0`, bytes `[123]` (`b'{'`).
Observed: fused SIMD returned `1`; scalar oracle expected `0`.

Cause: `next_special_avx2_all`/`next_special_ssse3_all`/`next_special_neon_all`
delegated short inputs to the core-only `next_inline_special_scalar`. The
all-marker tables were therefore bypassed whenever the remaining suffix was
shorter than the vector width.

Corrective diff applied to the final B patch:

```diff
 unsafe fn next_special_avx2_all(bytes: &[u8], from: usize) -> usize {
+    if bytes.len().saturating_sub(from) < 32 {
+        return next_inline_special_all_scalar(bytes, from);
+    }
     next_special_avx2_with_tables(bytes, from, &ALL_LOW_NIBBLE, &ALL_HIGH_NIBBLE)
 }
```

The equivalent guards use `< 16` for NEON and SSSE3. The full option-mask,
all-byte, offset, tail, and UTF-8 differential tests pass after this fix.
