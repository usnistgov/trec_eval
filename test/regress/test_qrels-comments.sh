# Every '#' comment line in a qrels, qrels_jg or prefs file is skipped,
# wherever it is. A file holding only comments is refused with exit 2.
. test/regress/lib.sh
D=test/regress/qrels-comments

# check FORMAT JUDGMENTS EXPECTED ARGS... (lib.sh uses the name "file")
check() {
    fmt=$1 judg=$2 exp=$3
    shift 3
    for v in end two fields; do
        expect_output $exp -q -R $fmt "$@" $judg.$v $D/run
        expect_status 0 -q -R $fmt "$@" $judg.$v $D/run
    done
    expect_status 2 -R $fmt "$@" $judg.only $D/run
    "$TREC_EVAL" -R $fmt "$@" $judg.only $D/run 2>&1 >/dev/null |
        grep -q "No data lines in .* file '$judg.only'" ||
        fail "trec_eval -R $fmt $judg.only: no message naming the file"
}

check qrels $D/qrels $D/out.num_rel -m num_rel
check qrels_jg $D/qrels $D/out.map_avgjg -m map_avgjg
check qrels_prefs $D/qrels $D/out.prefs -m prefs_num_prefs_poss
check prefs $D/prefs $D/out.prefs -m prefs_num_prefs_poss
exit 0
