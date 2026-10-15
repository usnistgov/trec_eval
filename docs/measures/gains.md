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

### 1. Exponential Gains (`ndcg_p`)
Computes NDCG using exponential gains $\text{gain}(r) = 2^r - 1$:

$$\text{gain}(r) = 2^r - 1$$

- Usage: `-m ndcg_p`

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
