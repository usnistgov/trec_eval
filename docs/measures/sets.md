# Set-Based Evaluation Measures

Set-based measures evaluate an unranked set of retrieved documents (such as in boolean retrieval, filtering, or classification tasks) where rank ordering within the set is ignored.

---

## Set Precision (`set_P`, `set_relative_P`)

Precision over the entire unranked retrieved set:

$$\text{set\_P} = \begin{cases} \frac{\text{num\_rel\_ret}}{\text{num\_ret}} & \text{if } \text{num\_ret} > 0 \\ 0.0 & \text{otherwise} \end{cases}$$

### Set Relative Precision (`set_relative_P`)
Normalizes set precision by the maximum possible precision given the number of relevant documents $R$ in the collection:

$$\text{set\_relative\_P} = \begin{cases} \frac{\text{set\_P}}{\min(1.0, R / \text{num\_ret})} & \text{if } \text{num\_ret} > 0 \text{ and } R > 0 \\ 0.0 & \text{otherwise} \end{cases}$$

---

## Set Recall (`set_recall`)

Recall over the entire unranked retrieved set:

$$\text{set\_recall} = \begin{cases} \frac{\text{num\_rel\_ret}}{R} & \text{if } R > 0 \\ 0.0 & \text{otherwise} \end{cases}$$

---

## Set MAP (`set_map`)

Average Precision over an unranked set. Assuming documents in the retrieved set are ordered uniformly at random, the expected AP is:

$$\text{set\_map} = \text{set\_P} \cdot \text{set\_recall} = \frac{(\text{num\_rel\_ret})^2}{\text{num\_ret} \cdot R}$$

---

## Set $F_\beta$-Measure (`set_F`)

Weighted harmonic mean of set precision and set recall. `set_F.x` uses the parameter $x$ itself, as `trec_eval`'s `m_set_F.c` does:

$$\text{set\_F}_x = \frac{(x + 1) \cdot \text{set\_P} \cdot \text{set\_recall}}{x \cdot \text{set\_P} + \text{set\_recall}}$$

The parameter $x$ is $\beta^2$ of the usual $F_\beta$, so `set_F.0.25` is $F_{0.5}$ and `set_F.4` is $F_2$.

- Parameter: $x$, the weight of recall against precision (default `1.0`, which gives $F_1$). It must be 0 or more, and `set_F.0` gives `set_P`.
- Usage: `-m set_F` ($F_1$), `-m set_F.0.25` ($F_{0.5}$, emphasizing precision) or `-m set_F.4` ($F_2$, emphasizing recall).
