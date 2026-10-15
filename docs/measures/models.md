# Summary Models, User Metrics & Diagnostics

---

## Geometric Mean Measures (`gm_map`, `gm_bpref`)

Geometric mean averaging across topic queries prevents systems from achieving high overall scores by performing well on a small number of easy queries while failing on difficult queries.

### Geometric Mean of MAP (`gm_map`)

For topic average precision scores $\text{AP}_q \in [0, 1]$:

$$\text{gm\_map} = \exp\left( \frac{1}{|Q|} \sum_{q \in Q} \ln\left( \max(\text{AP}_q, \epsilon) \right) \right)$$

where $\epsilon = 0.00001$ is the minimum score floor preventing $\ln(0)$.

- Usage: `-m gm_map`
- Reference: Robertson, S. (2006). "On GMAP: and other transformations of information retrieval metrics." *Proceedings of the 15th ACM CIKM*, pp. 78–83.

### Geometric Mean of Bpref (`gm_bpref`)

$$\text{gm\_bpref} = \exp\left( \frac{1}{|Q|} \sum_{q \in Q} \ln\left( \max(\text{bpref}_q, \epsilon) \right) \right)$$

---

## Rank-Biased Precision (`rbp`, `rbp_resid`)

Models a user browsing a ranked list who examines the document at rank $k$ with persistence probability $p^{k-1}$, stopping with probability $1 - p$.

### Definition

$$\text{RBP} = (1 - p) \sum_{k=1}^N \text{rel}(k) \cdot p^{k-1}$$

where $\text{rel}(k) \in [0, 1]$ is the normalized relevance grade ($\text{rel}(k) = \text{grade} / \text{max\_grade}$).

### RBP Residual (`rbp_resid`)

Upper bound on the potential unobserved score due to unjudged documents:

$$\text{RBP}_{\text{resid}} = \text{RBP} + p^N$$

- Parameter: `p` (persistence parameter, default `0.9`).
- Usage: `-m rbp` or `-m rbp.p=0.8`
- Reference: Moffat, A., & Zobel, J. (2008). "Rank-biased precision for measurement of retrieval effectiveness." *ACM Transactions on Information Systems (TOIS)*, 27(1): Article 2.

---

## 11-Point Interpolated Precision (`11pt_avg`, `iprec_at_recall`)

Interpolated precision evaluated at 11 standard recall levels: $0.0, 0.1, 0.2, \dots, 1.0$.

### Definition

Interpolated precision at recall level $r$:

$$P_{\text{interp}}(r) = \max_{r' \ge r} P(r')$$

- `11pt_avg`: Arithmetic mean of the 11 interpolated precision levels.
- `iprec_at_recall`: Reports the 11 individual interpolated precision scores.
- Usage: `-m 11pt_avg` or `-m iprec_at_recall`

---

## Linear Utility (`utility`)

Assigns explicit utility weights $a, b, c, d$ to the four cells of the $2 \times 2$ contingency table:

$$\text{Utility} = a \cdot \text{num\_rel\_ret} + b \cdot \text{num\_nonrel\_ret} + c \cdot \text{num\_rel\_nonret} + d \cdot \text{num\_nonrel\_nonret}$$

- Parameters: `utility.<a,b,c,d>` (default: `1.0, -1.0, 0.0, 0.0`).
- Requires `-N <num_docs_in_coll>` when $c \ne 0$ or $d \ne 0$.
- Usage: `-m utility.1,-1,0,0`

---

## Diagnostic Measures

### Relevance String (`relstring`)
Constructs a string representation of relevance judgments for the top $k$ retrieved documents (e.g. `'1-0-1-0-2'`).
- Parameter: cutoff length $k$ (default `10`).
- Usage: `-m relstring` or `-m relstring.20`

### Run Identifier (`runid`)
Reports the system run label recorded in column 6 of the run results file.
- Usage: `-m runid`
