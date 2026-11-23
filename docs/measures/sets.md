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

## Set $F$-Measure (`set_F`)

Weighted harmonic mean of set precision and set recall, parameterized by $x \ge 0$:

$$\text{set\_F}_x = \frac{x + 1}{\frac{1}{\text{set\_P}} + \frac{x}{\text{set\_recall}}} = \frac{(x + 1) \cdot \text{set\_P} \cdot \text{set\_recall}}{x \cdot \text{set\_P} + \text{set\_recall}}$$

Because `set_F` is a harmonic mean, $x$ is **not** a linear tradeoff between recall and precision; it weights $1/\text{set\_recall}$ relative to $1/\text{set\_P}$, so low values of either precision or recall continue to pull the score down strongly.

In van Rijsbergen's $E$-measure / $F_\beta$ formulation (*Information Retrieval*, 1979), $x = \beta^2$: $\beta = \sqrt{x}$ is the recall-to-precision ratio ($R/P$) at which small changes in precision and recall have equal effect on the score ($\partial F / \partial P = \partial F / \partial R$), and at $P = R$ the marginal sensitivity to recall is $x = \beta^2$ times that of precision.

- Parameter: $x \ge 0.0$ (default `1.0` for balanced $F_1$). At the boundary $x = 0.0$, $\text{set\_F}_0 = \text{set\_P}$; as $x \to \infty$, $\text{set\_F}_x \to \text{set\_recall}$.
- Usage: `-m set_F` ($F_1$, $x = 1.0$), `-m set_F.0.25` ($F_{0.5}$, $\beta = 0.5$, emphasizing precision), or `-m set_F.4.0` ($F_2$, $\beta = 2.0$, emphasizing recall).
