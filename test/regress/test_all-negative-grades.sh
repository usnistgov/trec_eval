# A topic whose grades are all negative has no relevant documents, so its
# average precision is 0. In both fixtures the other topic retrieves its
# one grade 1 document at rank 1 (AP 1), so map is (0 + 1) / 2 = 0.5.
. test/regress/lib.sh
d=test/regress/all-negative-grades

# The all-negative topic is evaluated first.
expect_status 0 -m map $d/qrels.first $d/results.first
expect_output $d/out.first -m map $d/qrels.first $d/results.first
expect_output $d/out.first.q -q -m map $d/qrels.first $d/results.first

# The all-negative topic follows a topic with grades 0 and 1.
expect_status 0 -m map $d/qrels.second $d/results.second
expect_output $d/out.second -m map $d/qrels.second $d/results.second
expect_output $d/out.second.q -q -m map $d/qrels.second $d/results.second
