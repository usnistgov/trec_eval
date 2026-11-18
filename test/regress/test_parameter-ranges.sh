# Measure parameters outside the range in which the measure is defined
# are refused when the measure is initialized, with exit status 2: recall
# levels outside [0, 1], multiples of R that are not positive, a negative
# set_F weight and an rbp persistence p outside [0, 1). They used to be
# accepted, and gave values such as iprec_at_recall_-1.00 0.4665.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test

# refused TEXT PARAMETER: exit status 2, and TEXT on standard error.
refused() {
    expect_status 2 -m "$2" $Q $R
    "$TREC_EVAL" -m "$2" $Q $R 2>&1 >/dev/null | grep -F -q "$1" ||
        fail "trec_eval -m $2: no message containing: $1"
}

refused "recall level -1 of measure 'iprec_at_recall' is not in [0, 1]" \
    iprec_at_recall.-1,.5
refused "recall level 1.5 of measure 'iprec_at_recall' is not in [0, 1]" \
    iprec_at_recall..5,1.5
refused "recall level -0.5 of measure '11pt_avg' is not in [0, 1]" \
    11pt_avg.-0.5,.5
refused "recall level 1.5 of measure '11pt_avg' is not in [0, 1]" \
    11pt_avg..5,1.5
refused "multiple 0 of R for measure 'Rprec_mult' is not positive" \
    Rprec_mult.0,1
refused "multiple -1 of R for measure 'Rprec_mult' is not positive" \
    Rprec_mult.-1
refused "multiple 0 of R for measure 'Rprec_mult_avgjg' is not positive" \
    Rprec_mult_avgjg.0
refused "weight -1 of R for measure 'set_F' is not 0 or more" set_F.-1
refused "persistence p=1 of measure 'rbp' is not in [0, 1)" rbp.p=1
refused "persistence p=-0.5 of measure 'rbp' is not in [0, 1)" rbp.p=-0.5
refused "persistence p=1.5 of measure 'rbp_resid' is not in [0, 1)" \
    rbp_resid.p=1.5

# The bounds that are part of each range, and the defaults, still work.
# 11pt_avg may still repeat a recall level, in any order.
for m in iprec_at_recall.0,1 iprec_at_recall 11pt_avg.1,0 11pt_avg \
         11pt_avg.0.5,0.5 \
         Rprec_mult.0.01 Rprec_mult set_F.0 set_F rbp.p=0 rbp \
         rbp_resid.p=0.99 rbp_resid; do
    expect_status 0 -m $m $Q $R
done
exit 0
