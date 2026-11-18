# 11pt_avg of a topic with nothing retrieved is 0, not nan. Under -J the
# unjudged documents x and y are removed, so topic 1 retrieves nothing;
# under -M 0 no topic retrieves anything.
. test/regress/lib.sh
d=test/regress/eleven-point-nan
expect_output $d/out.J -J -q -m 11pt_avg $d/qrels $d/results
expect_output $d/out.M0 -M 0 -q -m 11pt_avg $d/qrels $d/results
# iprec_at_recall has the same division; its cutoffs below the first relevant
# document took the nan. Topic 1 is 0 at every cutoff, topic 2 is 1.
expect_output $d/out.J.iprec -J -q -m iprec_at_recall $d/qrels $d/results
expect_output $d/out.M0.iprec -M 0 -q -m iprec_at_recall $d/qrels $d/results
