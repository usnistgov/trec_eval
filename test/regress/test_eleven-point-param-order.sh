# 11pt_avg averages the interpolated precision at the recall points given,
# in any order. The two relevant documents are at ranks 1 and 4 of 4, so
# the interpolated precision is 1 at recall 0.5 and 0.5 at recall 1.0,
# and the average is 0.75.
. test/regress/lib.sh
d=test/regress/eleven-point-param-order

expect_output $d/out.up -m 11pt_avg.0.5,1.0 $d/qrels $d/results
expect_output $d/out.down -m 11pt_avg.1.0,0.5 $d/qrels $d/results
