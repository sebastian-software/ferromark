# Candidate profile timings

Ratios compare each recipe with strict CommonMark on the same documents.
Features and HTML policy differ: this table is a cost budget, not a ranking
of equivalent renderers. Category values are unweighted geometric means
of per-document paired median ratios. Exact configurations are in the corpus.

| Candidate | Document group | Count | Fresh time ratio | Reused time ratio | Identical HTML / AST |
| --- | --- | ---: | ---: | ---: | ---: |
| commonmark | comments | 4 | 1.002× | 1.000× | 4/4 |
| commonmark | technical-docs | 2 | 0.998× | 0.999× | 2/2 |
| commonmark | readme | 1 | 1.003× | 0.992× | 1/1 |
| commonmark | reference | 2 | 1.003× | 1.007× | 2/2 |
| commonmark | encyclopedia | 3 | 1.001× | 1.004× | 3/3 |
| commonmark | plain-prose | 1 | 1.002× | 1.005× | 1/1 |
| gfm-spec | comments | 4 | 1.313× | 1.612× | 3/4 |
| gfm-spec | technical-docs | 2 | 1.107× | 1.127× | 2/2 |
| gfm-spec | readme | 1 | 1.177× | 1.184× | 0/1 |
| gfm-spec | reference | 2 | 1.837× | 1.864× | 1/2 |
| gfm-spec | encyclopedia | 3 | 1.100× | 1.118× | 3/3 |
| gfm-spec | plain-prose | 1 | 1.330× | 1.359× | 1/1 |
| comments | comments | 4 | 1.313× | 1.615× | 3/4 |
| comments | technical-docs | 2 | 1.165× | 1.197× | 2/2 |
| comments | readme | 1 | 1.252× | 1.255× | 0/1 |
| comments | reference | 2 | 2.054× | 2.090× | 0/2 |
| comments | encyclopedia | 3 | 1.347× | 1.360× | 3/3 |
| comments | plain-prose | 1 | 1.364× | 1.352× | 1/1 |
| article | comments | 4 | 1.027× | 1.042× | 3/4 |
| article | technical-docs | 2 | 1.083× | 1.100× | 0/2 |
| article | readme | 1 | 1.090× | 1.093× | 0/1 |
| article | reference | 2 | 1.024× | 1.047× | 0/2 |
| article | encyclopedia | 3 | 1.020× | 1.018× | 0/3 |
| article | plain-prose | 1 | 1.005× | 0.999× | 1/1 |
| docs | comments | 4 | 1.381× | 1.767× | 2/4 |
| docs | technical-docs | 2 | 1.253× | 1.312× | 0/2 |
| docs | readme | 1 | 1.340× | 1.355× | 0/1 |
| docs | reference | 2 | 1.938× | 1.937× | 0/2 |
| docs | encyclopedia | 3 | 1.144× | 1.157× | 0/3 |
| docs | plain-prose | 1 | 1.455× | 1.470× | 1/1 |

## Same-output parser ablations

Here only unused parser extensions are removed. Renderer policy is fixed,
and both HTML and AST must match exactly. Negative change means the
tailored recipe took less time than broad options on this input.

| Recipe | Input bytes | Broad fresh µs | Tailored fresh µs | Fresh change | Broad reuse µs | Tailored reuse µs | Reuse change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| comments | 345 | 2.995 | 2.574 | -14.1% | 2.559 | 2.112 | -17.5% |
| article | 4308 | 17.473 | 10.444 | -40.2% | 16.937 | 9.956 | -41.4% |
| docs | 65758 | 585.136 | 530.355 | -9.5% | 586.491 | 523.071 | -10.7% |
| mdx-docs | 4163 | 41.002 | 35.609 | -13.4% | 40.255 | 35.025 | -13.0% |
