# Longer confirmation and interaction runs

The selected main-study findings were remeasured with three process rounds,
three alternating pairs per round, and 50 ms windows. Percentages are time
changes, not throughput multipliers. All rows and ranges are in the adjacent
machine-readable result directories.

## Tailored parser configurations, identical output

| Recipe | Input bytes | Fresh time change | Reuse time change | Reuse change in each round |
| --- | ---: | ---: | ---: | --- |
| comments | 345 | -15.4% | -18.3% | -18.8%, -18.3%, -18.1% |
| article | 4308 | -39.8% | -41.2% | -40.9%, -41.2%, -41.2% |
| docs | 65758 | -9.7% | -9.4% | -9.4%, -9.7%, -10.0% |
| mdx-docs | 4163 | -13.5% | -13.6% | -12.7%, -13.7%, -13.5% |

## Renderer URL scanning after parser GFM autolinking

Parser GFM autolinks stay enabled. Only renderer `autolink_urls` is toggled;
link-target policies stay off. Every input preserves identical HTML and AST.
Positive changes are extra time from enabling the renderer pass. Custom URL
schemes or different output policies can require different behavior.

| Input | Actual bytes | Reuse time change | Change in each round |
| --- | ---: | ---: | --- |
| plain | 330 | +29.0% | +29.4%, +28.1%, +29.2% |
| urls | 354 | +5.7% | +5.7%, +5.5%, +5.5% |
| plain | 4125 | +13.0% | +13.0%, +13.0%, +13.3% |
| urls | 4130 | +3.9% | +3.4%, +3.7%, +4.7% |
| plain | 65670 | +12.0% | +12.0%, +12.9%, +11.4% |
| urls | 65549 | +4.4% | +4.6%, +4.4%, +4.2% |
