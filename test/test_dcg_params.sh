#!/bin/sh
# Regression tests for independent gain parameters on dcg, ideal_dcg, and ndcg (issue #195).
set -eu

BIN="${1:-./trec_eval}"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

run_eval() {
    desc="$1"; shift
    rc=0
    "$@" >"$TMPDIR/out" 2>"$TMPDIR/err" || rc=$?
    [ "$rc" -eq 0 ] || fail "$desc: expected exit 0, got $rc (stderr=$(cat "$TMPDIR/err"))"
    [ ! -s "$TMPDIR/err" ] || fail "$desc: unexpected stderr: $(cat "$TMPDIR/err")"
}

assert_val() {
    desc="$1"
    meas="$2"
    qid="$3"
    want="$4"
    got=$(awk -v m="$meas" -v q="$qid" '$1 == m && $2 == q { print $3 }' "$TMPDIR/out")
    [ "$got" = "$want" ] || fail "$desc: $meas ($qid) expected $want, got '${got:-<missing>}'\n$(cat "$TMPDIR/out")"
}

assert_exact() {
    desc="$1"
    expected="$2"
    printf '%s\n' "$expected" > "$TMPDIR/expected"
    diff -u "$TMPDIR/expected" "$TMPDIR/out" || fail "$desc: output mismatch"
}

# d1 has grade 1, d2 has grade 2, d3 has grade 0; run ranks d1, d2, d3.
# Default gains: dcg = 1 + 2/log2(3) = 2.2619, ideal_dcg = 2 + 1/log2(3) = 2.6309, ndcg = 0.8597.
# With 1=3: dcg = 3 + 2/log2(3) = 4.2619, ideal_dcg = 4.2619, ndcg = 1.0000.
# With 2=5: ideal_dcg = 5 + 1/log2(3) = 5.6309.
printf '1 0 d1 1\n1 0 d2 2\n1 0 d3 0\n' > "$TMPDIR/dcg.qrels"
printf '1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n1 Q0 d3 3 1 r\n' > "$TMPDIR/dcg.run"

# Baseline: all three without parameters
run_eval "unparameterized all three" "$BIN" -m dcg -m ideal_dcg -m ndcg "$TMPDIR/dcg.qrels" "$TMPDIR/dcg.run"
assert_exact "unparameterized all three" "$(printf 'dcg                   \tall\t2.2619\nideal_dcg             \tall\t2.6309\nndcg                  \tall\t0.8597')"

# Earlier measure has gain map, later measure does not (used to contaminate & double-free)
run_eval "dcg.1=3 with ideal_dcg" "$BIN" -m dcg.1=3 -m ideal_dcg "$TMPDIR/dcg.qrels" "$TMPDIR/dcg.run"
assert_exact "dcg.1=3 with ideal_dcg" "$(printf 'dcg_1=3               \tall\t4.2619\nideal_dcg             \tall\t2.6309')"

run_eval "dcg.1=3 with ideal_dcg and ndcg" "$BIN" -m dcg.1=3 -m ideal_dcg -m ndcg "$TMPDIR/dcg.qrels" "$TMPDIR/dcg.run"
assert_exact "dcg.1=3 with ideal_dcg and ndcg" "$(printf 'dcg_1=3               \tall\t4.2619\nideal_dcg             \tall\t2.6309\nndcg                  \tall\t0.8597')"

# Later measure has gain map, earlier measure does not (used to mis-score dcg & free string literal)
run_eval "ndcg.1=3 with dcg" "$BIN" -m ndcg.1=3 -m dcg "$TMPDIR/dcg.qrels" "$TMPDIR/dcg.run"
assert_exact "ndcg.1=3 with dcg" "$(printf 'dcg                   \tall\t2.2619\nndcg_1=3              \tall\t1.0000')"

run_eval "ndcg.1=3 with dcg and ideal_dcg" "$BIN" -m ndcg.1=3 -m dcg -m ideal_dcg "$TMPDIR/dcg.qrels" "$TMPDIR/dcg.run"
assert_exact "ndcg.1=3 with dcg and ideal_dcg" "$(printf 'dcg                   \tall\t2.2619\nideal_dcg             \tall\t2.6309\nndcg_1=3              \tall\t1.0000')"

# Distinct gain maps on all three measures (used to overwrite, leak, mis-score, and double-free)
run_eval "distinct params on all three" "$BIN" -m dcg.1=3 -m ideal_dcg.2=5 -m ndcg.1=3 "$TMPDIR/dcg.qrels" "$TMPDIR/dcg.run"
assert_exact "distinct params on all three" "$(printf 'dcg_1=3               \tall\t4.2619\nideal_dcg_2=5         \tall\t5.6309\nndcg_1=3              \tall\t1.0000')"

# Nickname all_trec (includes ndcg) combined with parameterized dcg
run_eval "all_trec with dcg.1=3" "$BIN" -m all_trec -m dcg.1=3 "$TMPDIR/dcg.qrels" "$TMPDIR/dcg.run"
assert_val "all_trec with dcg.1=3" dcg_1=3 all 4.2619
assert_val "all_trec with dcg.1=3" ndcg all 0.8597

# Per-query (-q) and no-summary (-n) modes
run_eval "per-query with distinct params" "$BIN" -q -n -m dcg.1=3 -m ideal_dcg.2=5 -m ndcg "$TMPDIR/dcg.qrels" "$TMPDIR/dcg.run"
assert_val "per-query with distinct params" dcg_1=3 1 4.2619
assert_val "per-query with distinct params" ideal_dcg_2=5 1 5.6309
assert_val "per-query with distinct params" ndcg 1 0.8597
