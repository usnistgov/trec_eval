# A run score that is not a number, or is NaN, is refused with a message
# naming the line. Characters after a number give a warning and the number
# is used. inf and -inf are accepted.
. test/regress/lib.sh
D=test/regress/run-score-validation

for r in nan abc; do
    expect_status 2 -q -m relstring $D/qrels $D/run.$r
    "$TREC_EVAL" -q -m relstring $D/qrels $D/run.$r 2>&1 >/dev/null |
        grep -q "Unusable score '$r' in line 2" ||
        fail "trec_eval $D/run.$r: no message naming the line"
done

expect_output $D/out.trail -q -m relstring $D/qrels $D/run.trail
"$TREC_EVAL" -q -m relstring $D/qrels $D/run.trail 2>&1 >/dev/null |
    grep -q "score '1.5x' in line 2 partially read as '1.5'" ||
    fail "trec_eval $D/run.trail: no warning"

expect_output $D/out.inf -q -m relstring $D/qrels $D/run.inf
exit 0
