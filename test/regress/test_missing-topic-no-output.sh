# Topic 2 of the qrels has no results. Without -c, trec_eval exits 4
# before it evaluates any topic, so -q prints no rows for topic 1. With
# -c, or with -D naming another topic, the output is as before.
. test/regress/lib.sh
D=test/regress/missing-topic-no-output

expect_status 4 -q -m num_ret $D/qrels $D/results
[ -z "$("$TREC_EVAL" -q -m num_ret $D/qrels $D/results 2>/dev/null)" ] ||
    fail "trec_eval -q -m num_ret $D/qrels $D/results: output on stdout"
"$TREC_EVAL" -q -m num_ret $D/qrels $D/results 2>&1 >/dev/null |
    diff - $D/err >/dev/null ||
    fail "trec_eval -q -m num_ret $D/qrels $D/results: stderr differs from $D/err"

expect_output $D/out.c -c -q -m num_ret $D/qrels $D/results
expect_status 0 -c -q -m num_ret $D/qrels $D/results
expect_output $D/out.D -D 0.1 -q -m num_ret $D/qrels $D/results
expect_status 0 -D 0.1 -q -m num_ret $D/qrels $D/results
exit 0
