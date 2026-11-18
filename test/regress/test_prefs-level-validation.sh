# The qrels_prefs and prefs readers refuse a relevance level that is not
# a number, or is NaN, with a message naming the document and topic. A
# level with characters after the number gives a warning, and the number
# is used. The .1 files give d1 the level 1; the .1x files must give the
# same values.
. test/regress/lib.sh
D=test/regress/prefs-level-validation
m="-m prefs_num_prefs_poss -m prefs_simp"

# check FORMAT PREFIX: read the files PREFIX.* with -R FORMAT
check() {
    for v in abc nan; do
        expect_status 2 -R $1 $m $D/$2.$v $D/results
        msg="Unusable relevance level '$v' of document d1 in topic 1"
        "$TREC_EVAL" -R $1 $m $D/$2.$v $D/results 2>&1 >/dev/null |
            grep -q "$msg" ||
            fail "-R $1 $D/$2.$v: no message naming the document"
    done
    expect_output $D/out.grade1 -R $1 $m $D/$2.1 $D/results
    expect_output $D/out.grade1 -R $1 $m $D/$2.1x $D/results
    "$TREC_EVAL" -R $1 $m $D/$2.1x $D/results 2>&1 >/dev/null |
        grep -q "level '1x' of document d1 in topic 1 partially read as '1'" ||
        fail "-R $1 $D/$2.1x: no warning"
}
check qrels_prefs qrels
check prefs prefs
