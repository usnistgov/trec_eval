# The help text gives the default of -N as 0, and the long form of -M
# with two dashes. Both statements are checked against the behaviour.
. test/regress/lib.sh
d=test/regress/help-text-defaults

help=$("$TREC_EVAL" -h)
n="^ -N <num>: Number of docs in collection. Default is 0.$"
echo "$help" | grep -q "$n" || fail "help text for -N"
echo "$help" | grep -q "^ --Max_retrieved_per_topic num:$" ||
    fail "help text for --Max_retrieved_per_topic"

# utility.1,-1,0,1 depends on -N; no -N gives the -N 0 values.
expect_output $d/out.N0 -q -m utility.1,-1,0,1 $d/qrels $d/results
expect_output $d/out.N0 -N 0 -q -m utility.1,-1,0,1 $d/qrels $d/results
# The long option of -M is accepted with two dashes.
m="-q -m num_ret $d/qrels $d/results"
expect_output $d/out.M1 --Max_retrieved_per_topic=1 $m
expect_output $d/out.M1 --Max_retrieved_per_topic 1 $m
