# Gain-Based & Discounted Measures

Gain-based measures like NDCG evaluate rankings using graded relevance judgments where higher relevance grades contribute greater utility, discounted by rank position.

---

## Discounted Cumulative Gain at Cutoffs (`ndcg_cut`, `ndcg`)

Normalized Discounted Cumulative Gain (NDCG) measures the ratio of actual Discounted Cumulative Gain to the ideal Discounted Cumulative Gain (IDCG) obtained by sorting all judged documents by relevance.

### Mathematical Formulation

$$\text{DCG}@k = \sum_{i=1}^k \frac{\text{gain}(\text{rel}_i)}{\log_2(i + 1)}$$

$$\text{NDCG}@k = \begin{cases} \frac{\text{DCG}@k}{\text{IDCG}@k} & \text{if } \text{IDCG}@k > 0 \\ 0.0 & \text{otherwise} \end{cases}$$

where:
- $\text{gain}(r)$ is the gain assigned to relevance grade $r$. In standard `trec_eval`, the default gain is the integer relevance level: $\text{gain}(r) = r$.
- $\text{IDCG}@k$ is the DCG computed on the ideal ranking formed by sorting all documents in the qrels by relevance grade in descending order.

### Cutoffs (`ndcg_cut`)
- Default rank cutoffs: $5, 10, 15, 20, 30, 100, 200, 500, 1000$.
- Usage: `-m ndcg_cut.10,20` or `ndcg@10`.

---

## NDCG Variants

### 1. Original Discount (`ndcg_p`)
Computes NDCG with the gains of `ndcg`, the relevance level by default, and the original discount of Järvelin and Kekäläinen (2002): rank 1 is not discounted, and rank $i \ge 2$ is divided by $\log_2 i$.

$$\text{DCG}_p = \text{gain}(\text{rel}_1) + \sum_{i=2}^{n} \frac{\text{gain}(\text{rel}_i)}{\log_2 i}$$

`ndcg` divides rank $i$ by $\log_2(i + 1)$, so the two differ from rank 2 on. For grades 1, 0, 2 at ranks 1 to 3, the ideal order is 2, 1, 0, and

$$\text{ndcg\_p} = \frac{1 + 2 / \log_2 3}{2 + 1} = 0.7540, \qquad \text{ndcg} = \frac{1 + 2 / \log_2 4}{2 + 1 / \log_2 3} = 0.7602.$$

Gains may be zero or negative, and level 0 may be given a gain. `ndcg_p` is 0 when the run retrieves no relevant document. `trec_eval` 10.x ships `m_ndcg_p.c` but does not register it; see [Differences from `trec_eval`](../cli.md#differences-from-trec_eval).

- Usage: `-m ndcg_p`, or `-m ndcg_p.1=3.5,2=9.0` for custom gains

### 2. Relative Relevance Level Gains (`ndcg_rel`)
Maps relevance grades to binary indicators relative to the relevance threshold cutoff $l$:

$$\text{gain}(r) = \begin{cases} 1.0 & \text{if } r \ge l \\ 0.0 & \text{otherwise} \end{cases}$$

- Usage: `-m ndcg_rel`

### 3. R-NDCG (`Rndcg`)
Computes NDCG evaluated at rank $R$, where $R$ is the total count of relevant documents in the collection:

$$\text{Rndcg} = \text{NDCG}@R$$

---

## Cumulative Gain (`G`, `binG`)

Evaluates undiscounted cumulative gain across the ranking.

### Cumulative Gain (`G`)
$$\text{G}@k = \sum_{i=1}^k \text{gain}(\text{rel}_i)$$

### Binary Cumulative Gain (`binG`)
$$\text{binG}@k = \sum_{i=1}^k \mathbb{I}(\text{rel}_i \ge l)$$

---

## Custom Relevance-to-Gain Mappings

Custom gain mappings can be passed globally via `--global-gains` (CLI) / `global_gains` (Python), or directly to the measure parameter string:

### Syntax
- CLI flag: `--global-gains 1=1.0,2=3.0,3=7.0,4=15.0`
- Metric parameter: `-m ndcg.1=1.0,2=3.0,3=7.0`
- Python API: `evaluator = Evaluator(qrels, measures=["ndcg@10"], global_gains="1=1.0,2=3.0")`
