# Measure Catalog Overview

`trec_eval` implements all 33 standard evaluation measures from reference `trec_eval 9.x/10.x`. Each measure is implemented according to its standard definition with explicit edge-case handling (empty rankings, zero relevant documents, unjudged documents).

All measures include unit tests as well as the `trec_eval` regression tests. Please let us know if you find an error or a discrepancy against the original codebase.

---

## Measure Specification Syntax

Measures can be requested on the CLI (`-m <name>`) or in Python (`measures=[...]`).

### 1. Default Metric Names
Requesting a measure by its root name calculates standard default parameters:
- `-m map`
- `-m Rprec`
- `-m recip_rank`

### 2. Parameterized Cutoffs (`.params`)
Cutoffs and parameters are specified following a period:
- `-m P.5,10,20` (Precision at ranks 5, 10, and 20)
- `-m ndcg_cut.10,20,100` (nDCG at cutoffs 10, 20, 100)
- `-m map_cut.1000` (MAP computed up to rank 1000)
- `-m rbp.p=0.85` (Rank-Biased Precision with persistence $p = 0.85$)
- `-m set_F.beta=0.5` ($F_\beta$-measure with $\beta = 0.5$)

### 3. Cutoff Aliases (`@cutoff`)
For cutoff measures, `@` syntax is supported as an alias:
- `ndcg@10` $\rightarrow$ `ndcg_cut.10`
- `P@5,10` $\rightarrow$ `P.5,10`
- `map@100` $\rightarrow$ `map_cut.100`
- `recall@10` $\rightarrow$ `recall.10`
- `mrr` $\rightarrow$ `recip_rank`


---

## Predefined Measure Groups

Groups expand into lists of standard measures:

### `official` (Default)
Standard TREC track summary report:
`runid`, `num_ret`, `num_rel`, `num_rel_ret`, `map`, `Rprec`, `recip_rank`, `bpref`, `P` (cutoffs 5, 10, 15, 20, 30, 100, 200, 500, 1000).

### `all_trec`
Full suite of standard measures:
`official` set plus `ndcg_cut`, `ndcg`, `recall`, `success`, `11pt_avg`, `utility`, `relstring`, `set_relative_P`, `set_map`, `set_F`.

### `set`
Set-based evaluation (unranked sets):
`runid`, `num_ret`, `num_rel`, `num_rel_ret`, `set_relative_P`, `set_map`, `set_F`.
