# Measure parameters that are NaN or infinite are refused with status 2:
# cutoffs, parameter lists and name=value pairs. Before, set_F.nan and
# set_F.inf printed nan, and a NaN cutoff was printed as a measure and
# escaped the cutoff sort and the duplicate check.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test

for m in iprec_at_recall.nan iprec_at_recall..5,nan,nan Rprec_mult.inf \
         11pt_avg.-inf set_F.nan set_F.inf relstring.infinity \
         utility.inf,-1,0,0 ndcg.1=nan rbp.p=inf rbp_resid.p=-inf; do
    expect_status 2 -m $m $Q $R
    "$TREC_EVAL" -m $m $Q $R 2>&1 >/dev/null |
        grep -q "is not a finite number" ||
        fail "trec_eval -m $m: no message"
done
exit 0
