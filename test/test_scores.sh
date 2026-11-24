#!/bin/sh
# Regression tests for float/double parsing (issue #107 and related parsers).
set -eu

BIN="${1:-./trec_eval}"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_fails() {
    desc="$1"; shift
    if "$@" >"$TMPDIR/out" 2>"$TMPDIR/err"; then
        fail "$desc: expected non-zero exit, got 0 (stdout=$(cat "$TMPDIR/out"))"
    fi
}

assert_succeeds() {
    desc="$1"; shift
    if ! "$@" >"$TMPDIR/out" 2>"$TMPDIR/err"; then
        fail "$desc: expected exit 0, got $? (stderr=$(cat "$TMPDIR/err"))"
    fi
}

# --- Issue #107: trec_results score parsing ---
printf '1 0 a 1\n1 0 b 2\n1 0 c 3\n' > "$TMPDIR/abc.qrels"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 4 r\n1 Q0 c 3 3 r\n' > "$TMPDIR/ok.run"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 nan r\n1 Q0 c 3 3 r\n' > "$TMPDIR/nan.run"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 NaN r\n1 Q0 c 3 3 r\n' > "$TMPDIR/nan_upper.run"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 -nan r\n1 Q0 c 3 3 r\n' > "$TMPDIR/neg_nan.run"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 abc r\n1 Q0 c 3 3 r\n' > "$TMPDIR/abc.run"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 1.5x r\n1 Q0 c 3 3 r\n' > "$TMPDIR/trail.run"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 inf r\n1 Q0 c 3 -inf r\n' > "$TMPDIR/inf.run"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 1e-400 r\n1 Q0 c 3 -1 r\n' > "$TMPDIR/underflow.run"
printf '1 Q0 a 1 5 r\n1 Q0 b 2 1e400 r\n1 Q0 c 3 3 r\n' > "$TMPDIR/overflow.run"

assert_succeeds "ok.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/ok.run"
grep -q "'123'" "$TMPDIR/out" || fail "ok.run expected '123', got: $(cat "$TMPDIR/out")"
[ ! -s "$TMPDIR/err" ] || fail "ok.run unexpected stderr: $(cat "$TMPDIR/err")"

assert_fails "nan.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/nan.run"
assert_fails "nan_upper.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/nan_upper.run"
assert_fails "neg_nan.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/neg_nan.run"
assert_fails "abc.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/abc.run"

assert_succeeds "trail.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/trail.run"
grep -q "'132'" "$TMPDIR/out" || fail "trail.run expected '132', got: $(cat "$TMPDIR/out")"
grep -q "Warning.*1.5x" "$TMPDIR/err" || fail "trail.run expected warning on stderr, got: $(cat "$TMPDIR/err")"

assert_succeeds "inf.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/inf.run"
grep -q "'213'" "$TMPDIR/out" || fail "inf.run expected '213', got: $(cat "$TMPDIR/out")"
[ ! -s "$TMPDIR/err" ] || fail "inf.run unexpected stderr: $(cat "$TMPDIR/err")"

assert_succeeds "underflow.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/underflow.run"
grep -q "'123'" "$TMPDIR/out" || fail "underflow.run expected '123', got: $(cat "$TMPDIR/out")"
[ ! -s "$TMPDIR/err" ] || fail "underflow.run unexpected stderr: $(cat "$TMPDIR/err")"

assert_succeeds "overflow.run" "$BIN" -q -m relstring "$TMPDIR/abc.qrels" "$TMPDIR/overflow.run"
grep -q "'213'" "$TMPDIR/out" || fail "overflow.run expected '213', got: $(cat "$TMPDIR/out")"
grep -q "Warning.*1e400" "$TMPDIR/err" || fail "overflow.run expected overflow warning, got: $(cat "$TMPDIR/err")"

# --- get_prefs.c ---
for bad in nan abc inf -inf; do
    printf '1 jg1 sub1 a 3.0\n1 jg1 sub1 b %s\n' "$bad" > "$TMPDIR/bad.prefs"
    assert_fails "prefs $bad" "$BIN" -m all_prefs -R prefs "$TMPDIR/bad.prefs" "$TMPDIR/ok.run"
done
printf '1 jg1 sub1 a 3.0\n1 jg1 sub1 b 1.5x\n' > "$TMPDIR/trail.prefs"
assert_succeeds "prefs 1.5x" "$BIN" -m all_prefs -R prefs "$TMPDIR/trail.prefs" "$TMPDIR/ok.run"
grep -q "Warning.*1.5x" "$TMPDIR/err" || fail "prefs 1.5x expected warning on stderr"

# --- get_qrels_prefs.c ---
for bad in nan abc inf; do
    printf '1 jg1 a 3.0\n1 jg1 b %s\n' "$bad" > "$TMPDIR/bad.qprefs"
    assert_fails "qrels_prefs $bad" "$BIN" -m all_prefs -R qrels_prefs "$TMPDIR/bad.qprefs" "$TMPDIR/ok.run"
done

# --- get_zscores.c ---
for bad in nan abc inf; do
    printf '1 map %s 0.1\n' "$bad" > "$TMPDIR/bad_mean.zscores"
    assert_fails "zscores mean $bad" "$BIN" -m map -Z "$TMPDIR/bad_mean.zscores" "$TMPDIR/abc.qrels" "$TMPDIR/ok.run"
    printf '1 map 0.5 %s\n' "$bad" > "$TMPDIR/bad_sd.zscores"
    assert_fails "zscores stddev $bad" "$BIN" -m map -Z "$TMPDIR/bad_sd.zscores" "$TMPDIR/abc.qrels" "$TMPDIR/ok.run"
done

# --- meas_init.c double cutoffs, double params, param pairs ---
for spec in \
    Rprec_mult.nan Rprec_mult.abc Rprec_mult.inf \
    set_F.nan set_F.abc set_F.inf \
    utility.1,nan,0,0 \
    ndcg.1=nan ndcg.1=abc ndcg.1=inf
do
    assert_fails "meas param $spec" "$BIN" -m "$spec" "$TMPDIR/abc.qrels" "$TMPDIR/ok.run"
done

assert_succeeds "meas param set_F.1.5x" "$BIN" -m set_F.1.5x "$TMPDIR/abc.qrels" "$TMPDIR/ok.run"
grep -q "Warning.*1.5x" "$TMPDIR/err" || fail "set_F.1.5x expected warning on stderr"
