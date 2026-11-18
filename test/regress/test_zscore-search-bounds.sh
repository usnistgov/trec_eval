# -Z prints -1000000 for a topic and measure that have no line in the
# zscores file. The zscores file has num_rel for topic 1 and num_ret for
# topic 2 only, and nothing for topic 3. Topic 1's num_ret must not be
# taken from topic 2's line, and the search for topic 3 must stay inside
# the table.
. test/regress/lib.sh
d=test/regress/zscore-search-bounds
expect_output $d/out -q -n -Z $d/zscores -m num_ret -m num_rel \
    $d/qrels $d/results
