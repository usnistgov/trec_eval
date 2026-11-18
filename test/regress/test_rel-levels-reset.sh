# Topic 1 has only negative grades, so it has no relevant document and
# its ideal DCG is 0. Its score must not depend on topic 0, which comes
# before it and has one document at grade 1. The gain map gives the
# negative-grade documents gain 1, so topic 1 has a nonzero DCG.
. test/regress/lib.sh
d=test/regress/rel-levels-reset

expect_output $d/out.ndcg.q -q -m ndcg.-2=1 $d/qrels $d/results
