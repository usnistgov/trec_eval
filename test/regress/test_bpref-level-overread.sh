# bpref and gm_bpref count the nonrelevant documents below the relevance
# level. A level above every grade must not read past the table of
# grades, and gives 0 because no document is relevant. The highest grade
# is 2 for topic 1 and 1 for topic 2.
. test/regress/lib.sh
d=test/regress/bpref-level-overread
expect_output $d/out.l1 -q -l 1 -m bpref -m gm_bpref $d/qrels $d/results
expect_output $d/out.l2 -q -l 2 -m bpref -m gm_bpref $d/qrels $d/results
expect_output $d/out.zero -q -l 3 -m bpref -m gm_bpref $d/qrels $d/results
expect_status 0 -l 1000000000000 -m bpref -m gm_bpref $d/qrels $d/results
expect_output $d/out.zero -q -l 1000000000000 -m bpref -m gm_bpref $d/qrels $d/results
