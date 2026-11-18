# Messages about a line of a results file give its line number in the
# file, comment and blank lines included. In each fixture, line 5 is the
# line at fault and follows a comment, a blank line and an indented
# comment.
. test/regress/lib.sh
D=test/regress/results-line-numbers

# check RUN STATUS MESSAGE
check() {
    run=$1 status=$2 msg=$3
    expect_status $status -m num_ret $D/qrels $run
    "$TREC_EVAL" -m num_ret $D/qrels $run 2>&1 >/dev/null |
        grep -q "$msg" ||
        fail "trec_eval $D/qrels $run: no message naming line 5"
}

check $D/run.malformed 2 "Malformed line 5\$"
check $D/run.score 2 "Unusable score 'abc' in line 5\$"
check $D/run.trail 0 "score '1.5x' in line 5 partially read as '1.5'\$"
exit 0
