#!/bin/sh
# Regression tests for -c (complete evaluation), -l (relevance level),
# -D (debug query), and empty-ranking handling (issues #95, #96, #142 item 4).
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

# --- Issue #95 minimal reproduction ---
printf '1 0 d1 1\n1 0 d2 2\n2 0 d1 2\n' > "$TMPDIR/numrel.qrels"
printf '1 Q0 d1 1 2 r\n1 Q0 d2 2 1 r\n2 Q0 d1 1 1 r\n' > "$TMPDIR/numrel.run"

run_eval "issue95 -l 2" "$BIN" -l 2 -q -m num_rel "$TMPDIR/numrel.qrels" "$TMPDIR/numrel.run"
assert_val "issue95 -l 2" num_rel 1 1
assert_val "issue95 -l 2" num_rel 2 1
assert_val "issue95 -l 2" num_rel all 2

run_eval "issue95 -c -l 2" "$BIN" -c -l 2 -q -m num_rel "$TMPDIR/numrel.qrels" "$TMPDIR/numrel.run"
assert_val "issue95 -c -l 2" num_rel 1 1
assert_val "issue95 -c -l 2" num_rel 2 1
assert_val "issue95 -c -l 2" num_rel all 2

run_eval "issue95 -c -l 2 -D1.1" "$BIN" -c -l 2 -q -D1.1 -m official "$TMPDIR/numrel.qrels" "$TMPDIR/numrel.run"
assert_val "issue95 -c -l 2 -D1.1" num_rel 1 1
assert_val "issue95 -c -l 2 -D1.1" num_rel all 1
assert_val "issue95 -c -l 2 -D1.1" num_q all 1
assert_val "issue95 -c -l 2 -D1.1" map all 0.5000
assert_val "issue95 -c -l 2 -D1.1" gm_map all 0.5000

# --- Multi-level qrels with missing topic (existing repo fixtures) ---
run_eval "rel_level trunc -c -l 2" "$BIN" -c -l 2 -q -m all_trec test/qrels.rel_level test/results.trunc
assert_val "rel_level trunc -c -l 2" num_rel 301 12
assert_val "rel_level trunc -c -l 2" num_rel 302 77
assert_val "rel_level trunc -c -l 2" num_rel 303 8
assert_val "rel_level trunc -c -l 2" num_rel all 97
assert_val "rel_level trunc -c -l 2" num_q all 3

# --- Missing FIRST topic (empty ranking when caches start empty) ---
printf '1 0 d1 1\n1 0 d2 2\n1 0 d3 -2\n2 0 d1 2\n2 0 d2 0\n3 0 d1 3\n' > "$TMPDIR/empty_first.qrels"
printf '2 Q0 d1 1 2.0 r\n' > "$TMPDIR/empty_first.run"

run_eval "missing first topic all_trec" "$BIN" -c -l 2 -q -m all_trec "$TMPDIR/empty_first.qrels" "$TMPDIR/empty_first.run"
assert_val "missing first topic all_trec" num_ret 1 0
assert_val "missing first topic all_trec" num_rel 1 1
assert_val "missing first topic all_trec" relstring 1 "''"
assert_val "missing first topic all_trec" iprec_at_recall_0.00 1 0.0000
assert_val "missing first topic all_trec" 11pt_avg 1 0.0000
assert_val "missing first topic all_trec" rbp_resid 1 0.0000
assert_val "missing first topic all_trec" unj_5 1 0.0000
assert_val "missing first topic all_trec" num_q all 3
assert_val "missing first topic all_trec" num_ret all 1
assert_val "missing first topic all_trec" num_rel all 3
assert_val "missing first topic all_trec" map all 0.3333

# --- -l 0 and negative -l exclude unjudged (-2) consistently ---
run_eval "missing first -c -l 0" "$BIN" -c -l 0 -q -m num_rel "$TMPDIR/empty_first.qrels" "$TMPDIR/empty_first.run"
assert_val "missing first -c -l 0" num_rel 1 2
assert_val "missing first -c -l 0" num_rel 2 2
assert_val "missing first -c -l 0" num_rel 3 1
assert_val "missing first -c -l 0" num_rel all 5

run_eval "missing first -c -l -1" "$BIN" -c -l -1 -q -m num_rel "$TMPDIR/empty_first.qrels" "$TMPDIR/empty_first.run"
assert_val "missing first -c -l -1" num_rel all 5

# --- Custom utility with non-zero p3 on empty ranking ---
run_eval "missing first utility" "$BIN" -c -l 2 -q -m utility.1,-1,-1,0 "$TMPDIR/empty_first.qrels" "$TMPDIR/empty_first.run"
assert_val "missing first utility" utility_1,-1,-1,0 1 -1.0000
assert_val "missing first utility" utility_1,-1,-1,0 2 1.0000
assert_val "missing first utility" utility_1,-1,-1,0 3 -1.0000
assert_val "missing first utility" utility_1,-1,-1,0 all -0.3333

# --- Missing first topic under qrels_jg ---
printf '1 jg1 d1 1\n1 jg1 d2 2\n1 jg2 d1 2\n2 jg1 d1 2\n2 jg1 d2 0\n' > "$TMPDIR/jg.qrels"
run_eval "missing first qrels_jg" "$BIN" -c -l 2 -q -R qrels_jg -m qrels_jg "$TMPDIR/jg.qrels" "$TMPDIR/empty_first.run"
assert_val "missing first qrels_jg" num_q all 2
assert_val "missing first qrels_jg" map_avgjg 1 0.0000
assert_val "missing first qrels_jg" map_avgjg 2 1.0000
assert_val "missing first qrels_jg" map_avgjg all 0.5000

# --- Missing first topic under prefs ---
printf '1 jg1 sub1 d1 2.0\n1 jg1 sub1 d2 0.0\n2 jg1 sub1 d1 2.0\n2 jg1 sub1 d2 0.0\n' > "$TMPDIR/p.prefs"
run_eval "missing first prefs" "$BIN" -c -q -R prefs -m all_prefs "$TMPDIR/p.prefs" "$TMPDIR/empty_first.run"
assert_val "missing first prefs" num_q all 2
assert_val "missing first prefs" prefs_num_prefs_poss all 2
assert_val "missing first prefs" prefs_simp all 0.5000

# --- -c with -Z preserves num_rel zscore accumulation and num_q count ---
run_eval "zscore -c" "$BIN" -c -q -m all_trec -Z test/zscores_file test/qrels.test test/results.trunc
assert_val "zscore -c" Znum_q all 3.0000
assert_val "zscore -c" Znum_rel all 0.0000

# --- Issue #96: parameterized measures and level -1 gains on missing topic ---
printf '1 0 a 1\n1 0 b 0\n2 0 a 1\n' > "$TMPDIR/two.qrels"
printf '1 Q0 a 1 2 r\n1 Q0 b 2 1 r\n' > "$TMPDIR/one.run"

run_eval "issue96 utility.1,-1,0,0" "$BIN" -c -q -m utility.1,-1,0,0 "$TMPDIR/two.qrels" "$TMPDIR/one.run"
assert_val "issue96 utility.1,-1,0,0" utility_1,-1,0,0 2 0.0000
assert_val "issue96 utility.1,-1,0,0" utility_1,-1,0,0 all 0.0000

run_eval "issue96 rbp_resid.p=0.5" "$BIN" -c -q -m rbp_resid.p=0.5 "$TMPDIR/two.qrels" "$TMPDIR/one.run"
assert_val "issue96 rbp_resid.p=0.5" rbp_resid_p=0.5 2 0.0000
assert_val "issue96 rbp_resid.p=0.5" rbp_resid_p=0.5 all 0.0000

run_eval "issue96 relstring.3" "$BIN" -c -q -m relstring.3 "$TMPDIR/two.qrels" "$TMPDIR/one.run"
assert_val "issue96 relstring.3" relstring_3 1 "'10'"
assert_val "issue96 relstring.3" relstring_3 2 "''"

run_eval "issue96 ndcg.-1=1" "$BIN" -c -q -m ndcg.-1=1 "$TMPDIR/two.qrels" "$TMPDIR/one.run"
assert_val "issue96 ndcg.-1=1" ndcg_-1=1 1 1.0000
assert_val "issue96 ndcg.-1=1" ndcg_-1=1 2 0.0000
assert_val "issue96 ndcg.-1=1" ndcg_-1=1 all 0.5000

run_eval "issue96 utility.1,-1,2,3" "$BIN" -c -q -m utility.1,-1,2,3 "$TMPDIR/two.qrels" "$TMPDIR/one.run"
assert_val "issue96 utility.1,-1,2,3" utility_1,-1,2,3 2 -1.0000
assert_val "issue96 utility.1,-1,2,3" utility_1,-1,2,3 all -3.5000

# --- Issue #142 item 4: -J (all unjudged) and -M 0 empty rankings avoid NaN ---
printf '1 0 a 1\n1 0 b 0\n' > "$TMPDIR/one.qrels"
printf '1 Q0 x 1 3 r\n1 Q0 y 2 2 r\n1 Q0 z 3 1 r\n' > "$TMPDIR/unjudged.run"

run_eval "issue142 -J all unjudged" "$BIN" -J -m iprec_at_recall -m 11pt_avg -m Rprec_mult "$TMPDIR/one.qrels" "$TMPDIR/unjudged.run"
assert_val "issue142 -J all unjudged" iprec_at_recall_0.00 all 0.0000
assert_val "issue142 -J all unjudged" 11pt_avg all 0.0000
assert_val "issue142 -J all unjudged" Rprec_mult_1.00 all 0.0000

run_eval "issue142 -M 0" "$BIN" -M 0 -m iprec_at_recall -m 11pt_avg -m Rprec_mult "$TMPDIR/one.qrels" "$TMPDIR/one.run"
assert_val "issue142 -M 0" iprec_at_recall_0.00 all 0.0000
assert_val "issue142 -M 0" 11pt_avg all 0.0000
assert_val "issue142 -M 0" Rprec_mult_1.00 all 0.0000
