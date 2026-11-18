# A results file that retrieves d1 twice for topic 1 cannot be evaluated:
# trec_eval exits 4, and the duplicate-docs message ends its own line
# before the "Can't calculate" message. The qrels, qrels_jg and
# qrels_prefs formats each detect the duplicate in their own code.
. test/regress/lib.sh
d=test/regress/duplicate-docs-newline

expect_stderr() {
    file=$1
    shift
    "$TREC_EVAL" "$@" 2>&1 >/dev/null | diff - "$file" >/dev/null ||
        fail "trec_eval $*: stderr differs from $file"
}

expect_status 4 -m map $d/qrels $d/results
expect_stderr $d/err -m map $d/qrels $d/results
expect_status 4 -R qrels_jg -m map_avgjg $d/qrels $d/results
expect_stderr $d/err.jg -R qrels_jg -m map_avgjg $d/qrels $d/results
expect_status 4 -R qrels_prefs -m prefs_num_prefs_poss $d/qrels $d/results
expect_stderr $d/err.prefs -R qrels_prefs -m prefs_num_prefs_poss $d/qrels $d/results
