#!/bin/sh
# Regression tests for distinct relevance-level storage, all-negative topics,
# fractional gain sorting, and related fixes (#94, #97, #98, #102, #142 item 9,
# #143 item 4, #144 item 2).
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
    if ! "$@" >"$TMPDIR/out" 2>"$TMPDIR/err"; then
        fail "$desc: expected exit 0, got $? (stderr=$(cat "$TMPDIR/err"))"
    fi
    if awk 'tolower($3) ~ /nan|inf/' "$TMPDIR/out" | grep -q .; then
        fail "$desc: unexpected NaN/Inf in output: $(awk 'tolower($3) ~ /nan|inf/' "$TMPDIR/out")"
    fi
}

assert_val() {
    desc="$1"
    meas="$2"
    qid="$3"
    expected="$4"
    actual=$(awk -v m="$meas" -v q="$qid" '$1 == m && $2 == q { print $3 }' "$TMPDIR/out")
    [ "$actual" = "$expected" ] || fail "$desc: $meas $qid expected '$expected', got '$actual'"
}

# --- Issue #97: topics whose qrels grades are all negative ---
printf '1 0 d1 -1\n1 0 d2 -1\n2 0 d1 0\n2 0 d2 1\n' > "$TMPDIR/neg-first.qrels"
printf '1 Q0 d1 1 2 r\n1 Q0 d2 2 1 r\n2 Q0 d2 1 2 r\n2 Q0 d1 2 1 r\n' > "$TMPDIR/neg-first.run"
printf '0 0 d1 0\n0 0 d2 1\n1 0 d1 -1\n1 0 d2 -1\n' > "$TMPDIR/neg-second.qrels"
printf '0 0 d1 0\n0 0 d2 1\n1 0 d1 -2\n1 0 d2 -2\n' > "$TMPDIR/neg2-second.qrels"
printf '0 Q0 d2 1 2 r\n0 Q0 d1 2 1 r\n1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n' > "$TMPDIR/neg-second.run"

run_eval "neg-first map" "$BIN" -q -m map "$TMPDIR/neg-first.qrels" "$TMPDIR/neg-first.run"
assert_val "neg-first map" map 1 0.0000
assert_val "neg-first map" map 2 1.0000
assert_val "neg-first map" map all 0.5000

run_eval "neg2-second map" "$BIN" -q -m map "$TMPDIR/neg2-second.qrels" "$TMPDIR/neg-second.run"
assert_val "neg2-second map" map 0 1.0000
assert_val "neg2-second map" map 1 0.0000
assert_val "neg2-second map" map all 0.5000

run_eval "neg-second map" "$BIN" -q -m map "$TMPDIR/neg-second.qrels" "$TMPDIR/neg-second.run"
assert_val "neg-second map" map 0 1.0000
assert_val "neg-second map" map 1 0.0000
assert_val "neg-second map" map all 0.5000

for m in ndcg ideal_dcg ndcg_rel G; do
    run_eval "neg-second $m.-2=1" "$BIN" -q -m "$m.-2=1" "$TMPDIR/neg-second.qrels" "$TMPDIR/neg-second.run"
    assert_val "neg-second $m.-2=1" "${m}_-2=1" 0 1.0000
    assert_val "neg-second $m.-2=1" "${m}_-2=1" 1 0.0000
    assert_val "neg-second $m.-2=1" "${m}_-2=1" all 0.5000
done

run_eval "neg-first ndcg_cut large cutoff" "$BIN" -q -m ndcg_cut.2147483647 "$TMPDIR/neg-first.qrels" "$TMPDIR/neg-first.run"
assert_val "neg-first ndcg_cut large cutoff" ndcg_cut_2147483647 1 0.0000
assert_val "neg-first ndcg_cut large cutoff" ndcg_cut_2147483647 2 1.0000
assert_val "neg-first ndcg_cut large cutoff" ndcg_cut_2147483647 all 0.5000

run_eval "neg-first all_trec" "$BIN" -q -m all_trec "$TMPDIR/neg-first.qrels" "$TMPDIR/neg-first.run"
assert_val "neg-first all_trec" ndcg 1 0.0000
assert_val "neg-first all_trec" rbp 1 0.0000

run_eval "neg2-second all_trec" "$BIN" -q -m all_trec "$TMPDIR/neg2-second.qrels" "$TMPDIR/neg-second.run"
assert_val "neg2-second all_trec" ndcg 1 0.0000
assert_val "neg2-second all_trec" rbp 1 0.0000

# --- Sparse/large relevance levels allocated by distinct count, not max_rel ---
printf '1 0 d1 0\n1 0 d2 1000000000\n1 0 d3 500000000\n' > "$TMPDIR/huge.qrels"
printf '1 Q0 d2 1 3 r\n1 Q0 d3 2 2 r\n1 Q0 d1 3 1 r\n' > "$TMPDIR/huge.run"

run_eval "sparse huge grades" "$BIN" -q -m map -m ndcg -m ndcg_cut.5 -m bpref -m gm_bpref "$TMPDIR/huge.qrels" "$TMPDIR/huge.run"
assert_val "sparse huge grades" map 1 1.0000
assert_val "sparse huge grades" ndcg 1 1.0000
assert_val "sparse huge grades" ndcg_cut_5 1 1.0000
assert_val "sparse huge grades" bpref 1 1.0000
assert_val "sparse huge grades" gm_bpref all 1.0000

# --- Negative gain override on a judged level ---
run_eval "negative gain override" "$BIN" -q -m ndcg.0=-1,1=2 "$TMPDIR/neg-second.qrels" "$TMPDIR/neg-second.run"
assert_val "negative gain override" ndcg_0=-1,1=2 0 0.6845
assert_val "negative gain override" ndcg_0=-1,1=2 1 0.0000
assert_val "negative gain override" ndcg_0=-1,1=2 all 0.3423

# --- bpref / gm_bpref when -l exceeds highest grade in topic (issue #102) ---
printf '1 0 a 2\n1 0 b 0\n1 0 c 1\n1 0 d 2\n2 0 a 1\n2 0 b 0\n' > "$TMPDIR/bpref.qrels"
printf '1 Q0 a 1 4 r\n1 Q0 c 2 3 r\n1 Q0 b 3 2 r\n1 Q0 d 4 1 r\n2 Q0 a 1 2 r\n2 Q0 b 2 1 r\n' > "$TMPDIR/bpref.run"

run_eval "issue102 -l 4" "$BIN" -q -l 4 -m bpref -m gm_bpref "$TMPDIR/bpref.qrels" "$TMPDIR/bpref.run"
assert_val "issue102 -l 4" bpref 1 0.0000
assert_val "issue102 -l 4" bpref 2 0.0000
assert_val "issue102 -l 4" bpref all 0.0000
assert_val "issue102 -l 4" gm_bpref all 0.0000

run_eval "issue102 -l 1000000000000" "$BIN" -l 1000000000000 -m bpref -m gm_bpref "$TMPDIR/bpref.qrels" "$TMPDIR/bpref.run"
assert_val "issue102 -l 1000000000000" bpref all 0.0000
assert_val "issue102 -l 1000000000000" gm_bpref all 0.0000

# --- LONG_MAX qrels grade (issue #143 item 4) ---
printf '1 0 d1 1\n1 0 d2 0\n1 0 d3 2\n2 0 d1 9223372036854775807\n' > "$TMPDIR/max.qrels"
printf '1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n1 Q0 d3 3 1 r\n2 Q0 d1 1 1 r\n2 Q0 d2 2 0 r\n' > "$TMPDIR/two.run"

run_eval "issue143 LONG_MAX map" "$BIN" -m map "$TMPDIR/max.qrels" "$TMPDIR/two.run"
assert_val "issue143 LONG_MAX map" map all 0.9167

run_eval "issue143 LONG_MAX qrels_jg" "$BIN" -R qrels_jg -m map_avgjg "$TMPDIR/max.qrels" "$TMPDIR/two.run"
assert_val "issue143 LONG_MAX qrels_jg" map_avgjg all 0.9167

# --- qrels_jg with all-negative JG and sparse large grade ---
printf '0 jg1 d1 0\n0 jg1 d2 1\n1 jg1 d1 -2\n1 jg1 d2 -2\n1 jg2 d1 100000000\n1 jg2 d2 0\n' > "$TMPDIR/jg.qrels"

run_eval "qrels_jg negative and sparse" "$BIN" -q -R qrels_jg -m qrels_jg "$TMPDIR/jg.qrels" "$TMPDIR/neg-second.run"
assert_val "qrels_jg negative and sparse" map_avgjg 0 1.0000
assert_val "qrels_jg negative and sparse" map_avgjg 1 0.5000
assert_val "qrels_jg negative and sparse" map_avgjg all 0.7500

# --- Issue #98: fractional gains < 1 apart and ndcg_rel zero ideal_dcg ---
printf '1 0 d1 1\n1 0 d2 2\n1 0 d3 3\n' > "$TMPDIR/gains.qrels"
printf '1 Q0 d2 1 3 r\n1 Q0 d3 2 2 r\n1 Q0 d1 3 1 r\n' > "$TMPDIR/gains.run"

run_eval "issue98 scaled gains" "$BIN" -m ndcg.1=0.3,2=0.6,3=0.9 "$TMPDIR/gains.qrels" "$TMPDIR/gains.run"
assert_val "issue98 scaled gains" ndcg_1=0.3,2=0.6,3=0.9 all 0.9225

run_eval "issue98 ideal order fractional" "$BIN" -m ndcg.0=0,1=0.3,2=0.9,3=0.6 "$TMPDIR/gains.qrels" "$TMPDIR/gains.run"
assert_val "issue98 ideal order fractional" ndcg_0=0,1=0.3,2=0.9,3=0.6 all 1.0000

run_eval "issue98 Rndcg fractional" "$BIN" -m Rndcg.0=0,1=0.3,2=0.9,3=0.6 "$TMPDIR/gains.qrels" "$TMPDIR/gains.run"
assert_val "issue98 Rndcg fractional" Rndcg_0=0,1=0.3,2=0.9,3=0.6 all 1.0000

run_eval "issue98 ndcg_rel fractional" "$BIN" -m ndcg_rel.0=0,1=0.3,2=0.9,3=0.6 "$TMPDIR/gains.qrels" "$TMPDIR/gains.run"
assert_val "issue98 ndcg_rel fractional" ndcg_rel_0=0,1=0.3,2=0.9,3=0.6 all 1.0000

printf '1 0 n -2\n1 0 a 2\n' > "$TMPDIR/neg_gain.qrels"
printf '1 Q0 n 1 2 r\n1 Q0 a 2 1 r\n' > "$TMPDIR/neg_gain.run"

run_eval "issue98 ndcg_rel zero ideal" "$BIN" -q -l2 -m ndcg_rel.-2=1,2=-3 "$TMPDIR/neg_gain.qrels" "$TMPDIR/neg_gain.run"
assert_val "issue98 ndcg_rel zero ideal" ndcg_rel_-2=1,2=-3 1 0.0000
assert_val "issue98 ndcg_rel zero ideal" ndcg_rel_-2=1,2=-3 all 0.0000

# --- Issue #142 item 9: Rndcg with num_rel > 0 and no positive gain ---
printf '1 0 d1 1\n1 0 d2 0\n1 0 d3 2\n2 0 d1 0\n2 0 d2 1\n' > "$TMPDIR/five.qrels"
printf '1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n1 Q0 d3 3 1 r\n2 Q0 d2 1 2 r\n2 Q0 d1 2 1 r\n' > "$TMPDIR/five.run"

run_eval "issue142 Rndcg.1=0" "$BIN" -q -m Rndcg.1=0 "$TMPDIR/five.qrels" "$TMPDIR/five.run"
assert_val "issue142 Rndcg.1=0" Rndcg_1=0 2 0.0000
assert_val "issue142 Rndcg.1=0" Rndcg_1=0 all 0.1250

printf '1 0 a 0\n1 0 b 0\n' > "$TMPDIR/allzero.qrels"
printf '1 Q0 a 1 2 r\n1 Q0 b 2 1 r\n' > "$TMPDIR/allzero.run"

run_eval "issue142 -l 0 Rndcg all-zero" "$BIN" -l 0 -q -m Rndcg "$TMPDIR/allzero.qrels" "$TMPDIR/allzero.run"
assert_val "issue142 -l 0 Rndcg all-zero" Rndcg 1 0.0000
assert_val "issue142 -l 0 Rndcg all-zero" Rndcg all 0.0000

# --- Issue #94: rbp initializes min/max deterministically ---
run_eval "issue94 rbp default" "$BIN" -m rbp test/qrels.test test/results.test
assert_val "issue94 rbp default" rbp all 0.3234

printf '1 0 a 1\n1 0 b 0\n' > "$TMPDIR/rbp.qrels"
printf '1 Q0 a 1 2 r\n1 Q0 b 2 1 r\n' > "$TMPDIR/rbp.run"

run_eval "issue94 rbp rescaled from 0" "$BIN" -m rbp.0=1,1=3 "$TMPDIR/rbp.qrels" "$TMPDIR/rbp.run"
assert_val "issue94 rbp rescaled from 0" rbp_0=1,1=3 all 0.1300
