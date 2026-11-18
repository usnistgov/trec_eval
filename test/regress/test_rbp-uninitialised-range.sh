# te_calc_rbp rescales gains to [0,1] using a range that starts at 0.
# With p = 0.9, rbp = 0.1 * sum over ranks of 0.9^(rank-1) * gain.
. test/regress/lib.sh
d=test/regress/rbp-uninitialised-range

# One relevant document at rank 1, default gains 0 and 1: 0.1 * 1.
expect_output $d/out.one -m rbp $d/qrels.one $d/results.one

# Gains 2 and 2 rescale to 1 and 1: 0.1 * (1 + 0.9 * 1) = 0.19.
expect_output $d/out.two.gains22 -m rbp.0=2,1=2 $d/qrels.two $d/results.two

# Gains 1 (grade 0) and 3 (grade 1) rescale to 1/3 and 1, and the grade 1
# document is at rank 1: 0.1 * (1 + 0.9 * 1/3) = 0.13.
expect_output $d/out.two.gains13 -m rbp.0=1,1=3 $d/qrels.two $d/results.two

# Topic 1 has grades 0-3 and topic 2 grades 0-2, so each topic's top
# document rescales to gain 1 and scores 0.1; the mean is 0.1. The range
# of topic 1 must not carry over to topic 2, with or without -q.
expect_output $d/out.topics -m rbp $d/qrels.topics $d/results.topics
expect_output $d/out.topics.q -q -m rbp $d/qrels.topics $d/results.topics
