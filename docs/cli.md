# Command-Line Interface Reference

`te-rust` is the standalone command-line evaluation tool. It accepts standard TREC relevance judgments (qrels) and system run results files, evaluates requested measures, and prints relational summary outputs.

It is a drop-in replacement for `trec_eval`, supporting the same command-line options and measure specifications. If you find an incompatibilty with trec_eval 10.x, that is a bug on our part

---

## Syntax

```bash
te-rust [OPTIONS] <QRELS_FILE> <RUN_FILE>
```

### Positional Arguments

| Argument | Description |
| :--- | :--- |
| `<QRELS_FILE>` | Path to the relevance judgments file. |
| `<RUN_FILE>` | Path to the system retrieval results (run) file. |

---

## Options and Flags

### Evaluation Flags

| Flag | Option | Description | Default |
| :--- | :--- | :--- | :--- |
| `-q` | | Print evaluations for each individual query/topic in addition to the summary averages. | Disabled |
| `-n` | | Suppress printing of the summary averages/totals at the end. | Disabled |
| `-l` | `<int>` | Minimum relevance level at which a document is considered relevant. Documents with judgments $< l$ are considered non-relevant. | `1` |
| `-c` | | **Complete set average**: Average over the complete set of queries present in the qrels file rather than the intersection of queries evaluated in the run. Topics missing from the run evaluate to 0.0. | Disabled |
| `-J` | | **Judged docs only**: Remove all unjudged documents from the retrieved ranking before evaluating. | Disabled |
| `-M` | `<int>` | Maximum number of retrieved documents per topic to evaluate. | No limit |
| `-N` | `<int>` | Number of documents in the collection (used for set-based recall and falloff calculations). | `0` |
| `-m` | `<measure>` | Calculate only the indicated measure or measure group (e.g. `-m map`, `-m P.5,10`, `-m official`). Repeatable. | `official` |
| | `--global-gains <gains>` | Custom global relevance-to-gain mapping (e.g. `--global-gains 1=3.5,2=9.0`). | Default gains |

### Bootstrap Confidence Interval Options

| Option | Description | Default |
| :--- | :--- | :--- |
| `-C`, `--ci` | Compute and print bootstrap confidence intervals for summary metrics in relational format (`measure_ci_lower`, `measure_ci_upper`). | Disabled |
| `--ci-pretty` | Format confidence intervals in bracketed human-readable format (`measure all value [lower, upper]`). | Disabled |
| `--ci-alpha <float>` | Significance level $\alpha$ for confidence intervals (e.g. `0.05` for a $95\%$ CI). | `0.05` |
| `--ci-samples <int>` | Number of bootstrap resamples. | `1000` |
| `--seed <int>` | Optional integer seed for deterministic, reproducible bootstrap resampling. | Random |

### Help and Discovery Options

| Option | Description |
| :--- | :--- |
| `--help-measures` | List all available measures with status and a short description, then exit. |
| `--help-measure <name>` | Print detailed mathematical explanation and parameter usage for the specified measure, then exit. |
| `-h`, `--help` | Print CLI usage summary and exit. |
| `-V`, `--version` | Print version information and exit. |

---

## File Formats

### 1. Relevance Judgments File (qrels)

Whitespace-delimited text file with 4 columns:

```text
qid  iter  docno  rel
```

- `qid`: Unique query/topic identifier (`str`).
- `iter`: Iteration/judgment tag (traditionally `0` or accessor ID; ignored during scoring).
- `docno`: Unique document identifier (`str`).
- `rel`: Integer relevance grade ($\ge 0$ for judged grades; negative integers reserved for unjudged pool sentinels).

Example `qrels.txt`:
```text
301 0 FBIS3-10082 1
301 0 FBIS3-10169 0
301 0 FBIS3-10243 2
302 0 LA010189-0001 1
```

Lines that start with `#` are comments and are ignored. Comments in qrels files were a trec_eval 10.x feature to support documenting how a qrels file should be use to reproduce published scores.

### 2. System Run File

Whitespace-delimited text file with 6 columns:

```text
qid  iter  docno  rank  sim  run_id
```

- `qid`: Unique query/topic identifier (`str`).
- `iter`: Iteration tag (traditionally `Q0`; ignored during scoring).
- `docno`: Unique document identifier (`str`).
- `rank`: Rank position (recomputed during score sorting).
- `sim`: Floating-point retrieval score / similarity (`float`).
- `run_id`: System run label (`str`).

Example `run.txt`:
```text
301 Q0 FBIS3-10082 1 14.567 my_bm25_run
301 Q0 FBIS3-10243 2 12.340 my_bm25_run
301 Q0 FBIS3-10169 3  9.812 my_bm25_run
```

Lines starting with `#` in either file are treated as comments and ignored. Comments in run files were a trec_eval 10.x feature to support including metadata and documentation about a run.

---

## Output Formats

### Standard Relational Output

Scores are printed as tab-separated triples: `measure  topic_id  score`.

```text
runid                 	all	my_bm25_run
num_q                 	all	50
num_ret               	all	50000
num_rel               	all	2279
num_rel_ret           	all	1613
map                   	all	0.2543
Rprec                 	all	0.2812
bpref                 	all	0.2589
recip_rank            	all	0.6124
P_5                   	all	0.4560
P_10                  	all	0.3980
```

When `-q` is enabled, per-topic evaluations precede the `all` summary block.

### Confidence Interval Output

With `--ci` relational output:
```text
map                   	all	0.2543
map_ci_lower          	all	0.2180
map_ci_upper          	all	0.2915
```

With `--ci-pretty` output:
```text
map                   	all	0.2543 [0.2180, 0.2915]
Rprec                 	all	0.2812 [0.2430, 0.3201]
bpref                 	all	0.2589 [0.2215, 0.2974]
```
