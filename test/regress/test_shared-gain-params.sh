# dcg, ideal_dcg and ndcg each read their own gain parameters. They
# used to share one parameter list: parameters given to one measure
# applied to the others, and the list was freed once per measure, which
# crashed trec_eval at exit. The qrels give d1 grade 1 and d2 grade 2,
# and the run ranks d1, d2, d3. With default gains, dcg is
# 1 + 2/log2(3) = 2.2619, ideal_dcg is 2 + 1/log2(3) = 2.6309 and ndcg
# is their ratio, 0.8597. Gain 3 for grade 1 gives dcg 3 + 2/log2(3) =
# 4.2619 and ndcg 1, because the run is then in ideal order. Gain 5 for
# grade 2 gives ideal_dcg 5 + 1/log2(3) = 5.6309.
. test/regress/lib.sh
d=test/regress/shared-gain-params

expect_status 0 -m dcg.1=3 -m ideal_dcg $d/qrels $d/results
expect_status 0 -m ndcg.1=3 -m dcg $d/qrels $d/results
expect_output $d/out.dcg -m dcg.1=3 -m ideal_dcg -m ndcg $d/qrels $d/results
expect_output $d/out.ndcg -m ndcg.1=3 -m dcg -m ideal_dcg $d/qrels $d/results
expect_output $d/out.each -m dcg.1=3 -m ideal_dcg.2=5 -m ndcg.1=3 \
    $d/qrels $d/results
exit 0
