# `te-rust` Predefined Measure Groups & Output Parity Design Document

This document defines the design and specifications for bringing `te-rust` predefined measure groups (`all_trec`, `official`, `set`, `qrels_jg`) and output format into exact parity with C `trec_eval` (`trec_eval/measures.c`).

---

## 1. Overview and Problem Statement

When running `trec_eval` and `te-rust` on standard test runs, diff comparisons revealed several discrepancies:
1. **Missing measures in predefined groups**: `expand_group` in `registry.rs` omitted 18 measures from `all_trec`, 3 measures from `official`, 4 measures from `set`, and `runid`/`num_q` from `qrels_jg`.
2. **Missing `num_q` measure**: `num_q` was not registered in the metric registry, leaving it missing from summary outputs.
3. **Ordering in predefined groups**: When predefined groups are expanded, their member lists should follow the canonical C `trec_eval` definition order (`te_trec_measures[]`).
4. **Command-line flexibility**: Individual `-m` flags passed on the CLI continue to be evaluated in user-specified insertion order.

---

## 2. Canonical Measure List & Group Definitions

In C `trec_eval` (`measures.c`), the canonical declaration order of measures is:

1. `runid`
2. `num_q`
3. `num_ret`
4. `num_rel`
5. `num_rel_ret`
6. `map`
7. `gm_map`
8. `Rprec`
9. `bpref`
10. `recip_rank`
11. `iprec_at_recall`
12. `P`
13. `relstring`
14. `recall`
15. `infAP`
16. `gm_bpref`
17. `Rprec_mult`
18. `utility`
19. `11pt_avg`
20. `binG`
21. `G`
22. `ndcg`
23. `ndcg_rel`
24. `Rndcg`
25. `ndcg_cut`
26. `map_cut`
27. `relative_P`
28. `success`
29. `set_P`
30. `set_relative_P`
31. `set_recall`
32. `set_map`
33. `set_F`
34. `num_nonrel_judged_ret`
35. `map_avgjg`
36. `P_avgjg`
37. `Rprec_mult_avgjg`
38. `yaap`
39. `rbp`
40. `rbp_resid`
41. `unj`

### Group Expansion Mapping

* **`official`** (12 measures):
  `["runid", "num_q", "num_ret", "num_rel", "num_rel_ret", "map", "gm_map", "Rprec", "bpref", "recip_rank", "iprec_at_recall", "P"]`

* **`all_trec`** (37 measures):
  `["runid", "num_q", "num_ret", "num_rel", "num_rel_ret", "map", "gm_map", "Rprec", "bpref", "recip_rank", "iprec_at_recall", "P", "relstring", "recall", "infAP", "gm_bpref", "Rprec_mult", "utility", "11pt_avg", "binG", "G", "ndcg", "ndcg_rel", "Rndcg", "ndcg_cut", "map_cut", "relative_P", "success", "set_P", "set_relative_P", "set_recall", "set_map", "set_F", "num_nonrel_judged_ret", "rbp", "rbp_resid", "unj"]`

* **`set`** (11 measures):
  `["runid", "num_q", "num_ret", "num_rel", "num_rel_ret", "utility", "set_P", "set_relative_P", "set_recall", "set_map", "set_F"]`

* **`qrels_jg`** (5 measures):
  `["runid", "num_q", "map_avgjg", "P_avgjg", "Rprec_mult_avgjg"]`

---

## 3. `NumQMeasure` Design

`num_q` is a standard metadata measure that counts the number of queries evaluated.

* **Module**: `te_rust::metrics::num_q`
* **Format**: `ValueFormat::Integer`
* **Evaluation Type**: `EvaluationType::Standard`
* **Query output**: `is_query_enabled(&self) -> bool { false }` (C's `te_print_single_meas_empty`).
* **Summary output**: `is_summary_enabled(&self) -> bool { true }`.
* **Summary value calculation**:
  * If `config.average_complete_flag` is true (`-c`), the value is `total_qrels_queries`.
  * Otherwise, the value is `num_queries_evaluated`.
* **Registry position**: Canonical index 1 (directly after `runid`).

---

## 4. Testing Strategy

1. **Unit Tests**:
   * Test `NumQMeasure` for both standard and `-c` (average_complete) configurations.
   * Verify group expansions match exact expected lists and order.
2. **Regression Suite**:
   * Verify all standard regression tests pass with 0 epsilon difference against C `trec_eval`.
   * Compare full `all_trec` and `official` outputs on multi-topic datasets against live C execution.
