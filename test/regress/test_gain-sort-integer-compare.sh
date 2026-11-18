# The gain table used by ndcg must be sorted by gain even when gains
# differ by less than 1. The run ranks d2, d3, d1 (grades 2, 3, 1).
. test/regress/lib.sh
d=test/regress/gain-sort-integer-compare
# Gains 0.9, 0.6, 0.3 in rank order: the ranking is ideal.
expect_output $d/out.ideal -q -m ndcg.0=0,1=0.3,2=0.9,3=0.6 $d/qrels $d/results
# Gains are 0.3 times the default gains, so ndcg equals the default 0.9225.
expect_output $d/out.scaled -q -m ndcg.1=0.3,2=0.6,3=0.9 $d/qrels $d/results
