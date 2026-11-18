# A name=value parameter list that gives the same relevance level twice,
# or the same name twice, is refused with exit status 2. Before, the
# measure used one of the two gains for the run and could use the other
# for the ideal ranking: ndcg.1=5,1=2 printed 0.1608, which is
# 2/5 of the 0.4021 that any one positive gain for level 1 gives here.
. test/regress/lib.sh
D=test/regress/repeated-gain-levels
Q=test/qrels.test
R=test/results.test

# refused TEXT PARAMETER: exit status 2, and TEXT on standard error.
refused() {
    expect_status 2 -m "$2" $Q $R
    "$TREC_EVAL" -m "$2" $Q $R 2>&1 >/dev/null | grep -F -q "$1" ||
        fail "trec_eval -m $2: no message containing: $1"
}

for m in ndcg.1=2,1=5 ndcg.1=5,1=2 ndcg.1=2,01=5 ndcg.2=1,1=2,+1=3; do
    refused "relevance level 1 of measure 'ndcg' is given more than once" $m
done
refused "relevance level 1 of measure 'G' is given more than once" G.1=2,1=3
refused "parameter 'p' of measure 'rbp' is given more than once" \
    rbp.p=0.5,p=0.8

# Distinct levels, and p beside a level, are accepted as before.
expect_output $D/out.distinct -m ndcg.0=1,1=3 -m G.0=1,1=3 $Q $R
expect_status 0 -m rbp.p=0.5,2=3 $Q $R
exit 0
