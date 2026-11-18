# A malformed line in a qrels, qrels_jg or prefs file is reported by its
# line number in the file, comment lines included. In each fixture, line 4
# has no relevance field and follows two comment lines.
. test/regress/lib.sh
D=test/regress/qrels-line-numbers

# check FORMAT JUDGMENTS ARGS...
check() {
    fmt=$1 judg=$2
    shift 2
    expect_status 2 -R $fmt "$@" $judg $D/run
    "$TREC_EVAL" -R $fmt "$@" $judg $D/run 2>&1 >/dev/null |
        grep -q "Malformed line 4\$" ||
        fail "trec_eval -R $fmt $judg: no message naming line 4"
}

check qrels $D/qrels -m num_rel
check qrels_jg $D/qrels -m map_avgjg
check qrels_prefs $D/qrels -m prefs_num_prefs_poss
check prefs $D/prefs -m prefs_num_prefs_poss
exit 0
