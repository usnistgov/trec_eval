# -M sets the number of documents evaluated per topic. Topic 1 retrieves
# three documents and topic 2 two.
. test/regress/lib.sh
d=test/regress/negative-max-docs

# A negative -M is refused like an unparsable one: exit 1, no output.
expect_status 1 -M -1 -m num_ret -q $d/qrels $d/results
expect_output $d/out.empty -M -1 -m num_ret -q $d/qrels $d/results
err=$("$TREC_EVAL" -M -1 -m num_ret -q $d/qrels $d/results 2>&1 >/dev/null)
[ "$err" = "trec_eval: Invalid maximum number of documents per topic '-1'" ] ||
    fail "-M -1 printed '$err'"

# -M 0 evaluates no documents; -M 2 keeps two documents of each topic.
expect_output $d/out.M0 -M 0 -m num_ret -q $d/qrels $d/results
expect_output $d/out.M2 -M 2 -m num_ret -q $d/qrels $d/results
