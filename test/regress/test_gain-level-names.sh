# A name=value parameter whose name is not a relevance level is refused
# when the measure is initialised, with exit status 2, for every measure
# with such parameters: dcg, ideal_dcg, ndcg, ndcg_rel, Rndcg, G, rbp and
# rbp_resid. rbp and rbp_resid still read p. Before, the pair was skipped
# without a message.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test

# refused NAME PARAMETER: exit status 2, and on standard error the message
# for NAME and then the one for the measure of PARAMETER.
refused() {
    expect_status 2 -m "$2" $Q $R
    err=$("$TREC_EVAL" -m "$2" $Q $R 2>&1 >/dev/null)
    case $err in
    *"Unusable relevance level '$1'"*"Cannot initialize measure '${2%%.*}'"*) ;;
    *) fail "trec_eval -m $2: wrong messages: $err" ;;
    esac
}

refused x ndcg.x=1
refused x ndcg.1=3,x=1
refused p ndcg.p=0.5
refused 1x ndcg.1x=2
refused '' ndcg.=1
refused 99999999999999999999 ndcg.99999999999999999999=1
for m in dcg ideal_dcg ndcg_rel Rndcg G rbp rbp_resid; do
    refused x $m.x=1
done
refused q rbp.q=0.5
refused q rbp_resid.p=0.5,q=1

for m in ndcg.+1=2 rbp.p=0.5 rbp.p=0.5,2=3 rbp_resid.p=0.5 \
         rbp_resid.p=0.5,2=3; do
    expect_status 0 -m $m $Q $R
done
exit 0
