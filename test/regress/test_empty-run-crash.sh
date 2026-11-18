# A results file with no data lines is refused with exit 2 and a message
# naming the file; it used to crash.
. test/regress/lib.sh
D=test/regress/empty-run-crash

for run in $D/run.blank $D/run.comment; do
    expect_status 2 $D/qrels $run
    "$TREC_EVAL" $D/qrels $run 2>&1 >/dev/null |
        grep -q "No data lines in results file '$run'" ||
        fail "trec_eval $D/qrels $run: no message naming the file"
done
expect_status 2 $D/qrels - </dev/null
exit 0
