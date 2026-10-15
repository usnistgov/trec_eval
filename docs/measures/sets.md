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

Weighted harmonic mean of set precision and set recall, parameterized by $\beta$:

$$F_\beta = \frac{(1 + \beta^2) \cdot \text{set\_P} \cdot \text{set\_recall}}{\beta^2 \cdot \text{set\_P} + \text{set\_recall}}$$

- Parameter: `beta` (default `1.0` for balanced $F_1$).
- Usage: `-m set_F` ($F_1$) or `-m set_F.beta=0.5` ($F_{0.5}$, emphasizing precision) or `-m set_F.beta=2.0` ($F_2$, emphasizing recall).
