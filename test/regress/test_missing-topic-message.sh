# Topic 2 of the qrels has no results. Without -c, trec_eval exits 4 with
# a message naming that qrels topic. A results topic that the qrels lack
# (topic 3 of results.extra) is skipped without a message.
. test/regress/lib.sh
D=test/regress/missing-topic-message

expect_status 4 -m num_ret $D/qrels $D/results
"$TREC_EVAL" -m num_ret $D/qrels $D/results 2>&1 >/dev/null |
    diff - $D/err >/dev/null ||
    fail "trec_eval -m num_ret $D/qrels $D/results: stderr differs from $D/err"

expect_status 0 -m num_ret $D/qrels $D/results.extra
[ -z "$("$TREC_EVAL" -m num_ret $D/qrels $D/results.extra 2>&1 >/dev/null)" ] ||
    fail "trec_eval -m num_ret $D/qrels $D/results.extra: unexpected message"
exit 0
