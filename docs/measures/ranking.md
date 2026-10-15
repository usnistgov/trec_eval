# Standard Ranking Measures

---

## Mean Average Precision (`map`, `map_cut`)

Average Precision (AP) rewards systems that retrieve relevant documents early in the ranking.

### Definition

For a query topic with $R$ total relevant documents in the collection:

$$\text{AP} = \frac{1}{R} \sum_{k=1}^{N} \text{rel}(k) \cdot P@k$$

where:
- $\text{rel}(k) = 1$ if the document at rank $k$ is relevant ($\text{relevance} \ge l$), and $0$ otherwise.
- $P@k = \frac{\sum_{i=1}^k \text{rel}(i)}{k}$ is the precision at rank $k$.
- $N$ is the number of retrieved documents evaluated (up to cutoff $C$ for `map_cut`).

### Mean Across Topics (MAP)

$$\text{MAP} = \frac{1}{|Q|} \sum_{q \in Q} \text{AP}_q$$

### Edge Cases
- If $R = 0$ (no relevant documents for topic in judgments): $\text{AP} = 0.0$.
- If no relevant documents are retrieved: $\text{AP} = 0.0$.
- Under `-c` / `complete_set_average`, queries missing from the run evaluate to $0.0$ and are included in the denominator $|Q_{\text{qrels}}|$.

---

## Precision at Cutoffs (`P`, `relative_P`)

Precision measures the fraction of retrieved documents that are relevant within the top $k$ ranks.

### Definition

$$P@k = \frac{\text{relevant documents in top } k}{k}$$

- Default cutoffs: $5, 10, 15, 20, 30, 100, 200, 500, 1000$.
- Usage: `-m P.5,10,20` or `P@10`.

### Relative Precision (`relative_P`)

Relative precision normalizes $P@k$ by the maximum achievable precision at cutoff $k$, $\min(1.0, R / k)$:

$$\text{rel\_P}@k = \frac{P@k}{\min(1.0, R / k)}$$

---

## R-Precision (`Rprec`, `Rprec_mult`)

Precision evaluated at rank $R$, where $R$ is the total number of relevant documents for that query in the collection.

### Definition

$$\text{Rprec} = P@R = \frac{\text{relevant documents in top } R}{R}$$

### Multiples of R (`Rprec_mult`)

Evaluates precision at rank $\lfloor c \cdot R + 0.5 \rfloor$ for multiplier $c$:

$$\text{Rprec\_mult}@c = \frac{\text{relevant documents in top } \lfloor c \cdot R + 0.5 \rfloor}{R}$$

- Default multipliers: $0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0$.

---

## Reciprocal Rank (`recip_rank`)

Measures the reciprocal of the rank at which the first relevant document is retrieved.

### Definition

$$\text{RR} = \begin{cases} \frac{1}{\text{rank}_1} & \text{if a relevant document is retrieved at rank}_1 \\ 0.0 & \text{otherwise} \end{cases}$$

The mean across all queries is Mean Reciprocal Rank (MRR).

---

## Binary Preference (`bpref`)

Measures retrieval performance using only judged documents, designed to be robust to incomplete judgment pools.

### Definition

Let $R$ be the number of relevant documents for the topic, and $r$ be a retrieved relevant document. Let $n$ be a retrieved non-relevant document ranked ahead of $r$.

$$\text{bpref} = \frac{1}{R} \sum_{r} \left(1 - \frac{\min(R, \text{number of non-relevant retrieved ahead of } r)}{\min(R, N_{\text{nonrel}})}\right)$$

- Reference: Buckley, C., & Voorhees, E. M. (2004). "Retrieval evaluation with incomplete information." *Proceedings of the 27th ACM SIGIR*, pp. 25–32.

---

## Inferred Average Precision (`infap`)

Estimates Average Precision when judgment pools are incomplete and documents are sampled for judgment with non-uniform inclusion probabilities.

- Reference: Yilmaz, E., & Aslam, J. A. (2006). "Estimating average precision with incomplete and imperfect judgments." *Proceedings of the 15th ACM CIKM*, pp. 102–111.

---

## Recall at Cutoffs (`recall`)

Fraction of all relevant documents retrieved within the top $k$ ranks:

$$\text{Recall}@k = \frac{\text{relevant documents in top } k}{R}$$

---

## Success at Cutoffs (`success`)

Binary indicator whether at least one relevant document was found within the top $k$ ranks:

$$\text{Success}@k = \begin{cases} 1.0 & \text{if } \sum_{i=1}^k \text{rel}(i) \ge 1 \\ 0.0 & \text{otherwise} \end{cases}$$

- Default cutoffs: $1, 5, 10$.

---

## Unjudged Fraction (`unj`)

Fraction of retrieved documents in the top $k$ ranks that are not present in the judgment pool:

$$\text{unj}@k = \frac{\text{unjudged documents in top } k}{k}$$

- Default cutoffs: $5, 10, 20$.
