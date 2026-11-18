# utility takes exactly four coefficients. Another number is refused when
# the measure is initialised, with exit 2 and no output, as for other
# unusable measure parameters. Before, it was refused only while scoring
# the first topic, with exit 4.
. test/regress/lib.sh
d=test/regress/utility-coefficient-count

# check PARAMS COUNT
check() {
    expect_status 2 -q -m utility.$1 $d/qrels $d/results
    expect_output $d/out.empty -q -m utility.$1 $d/qrels $d/results
    "$TREC_EVAL" -q -m utility.$1 $d/qrels $d/results 2>&1 >/dev/null |
        grep -q "improper number of coefficients: $2 given, 4 needed" ||
        fail "-m utility.$1: no message naming the count"
}
check 1,2 2
check 1,2,3,4,5 5

expect_output $d/out.valid -q -m utility.2,-1,0,0 $d/qrels $d/results
