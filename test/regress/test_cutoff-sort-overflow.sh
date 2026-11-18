# Cutoffs that differ by 2^31 or more must still be sorted in increasing
# order. The fixture ranks a (grade 1), b (grade 0) and c (grade 1), so
# P_1 = 1, recall_1 = recall_2 = 1/2, and every cutoff above 3 has
# P = 2 / cutoff (0.0000 at four decimals) and recall 1.
. test/regress/lib.sh
d=test/regress/cutoff-sort-overflow

expect_output $d/out.P -m P.4294967297,1 $d/qrels $d/results
expect_output $d/out.recall -m recall.2,4294967297,1 $d/qrels $d/results
