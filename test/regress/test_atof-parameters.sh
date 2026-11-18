# Floating-point measure parameters that are not numbers are refused, as
# integer cutoffs are (-m P.x); they used to be read as 0.
. test/regress/lib.sh
D=test/regress/atof-parameters

for m in iprec_at_recall.x Rprec_mult.x set_F.x 11pt_avg.x \
         iprec_at_recall..1,,.2 ndcg.1=x rbp.p=x; do
    expect_status 2 -m $m $D/qrels $D/run
done

# Values such as .25, 1. and 0.5 still parse; gain pairs still work.
expect_output $D/out.valid -m iprec_at_recall..25,1.,0.5 \
    -m Rprec_mult.1.,.5 -m set_F.0.5 -m 11pt_avg.0.5,1. \
    -m ndcg.1=3,2=9,4=4.5 $D/qrels $D/run

# Trailing characters after a number give a warning; the number is used.
expect_output $D/out.partial -m iprec_at_recall..25x $D/qrels $D/run
"$TREC_EVAL" -m iprec_at_recall..25x $D/qrels $D/run 2>&1 >/dev/null |
    grep -q "partially read as" ||
    fail "trec_eval -m iprec_at_recall..25x: no warning"
exit 0
