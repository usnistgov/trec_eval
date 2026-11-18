# A run score outside the range of a double is kept as strtod reads it,
# +-inf or +-0, with a warning that names the line. Scores that become
# equal tie, and the tie is broken by docno: b, a, c for 1e401, 1e400 and
# -1e400, and c, b, a for 2e-400, 1e-400 and -1e-400. A subnormal score
# is kept without a warning. The line number counts comment lines, as in
# the other results messages.
. test/regress/lib.sh
D=test/regress/run-score-range

# warns RUN TEXT: the warnings for RUN include TEXT.
warns() {
    "$TREC_EVAL" -q -m relstring $D/qrels $D/$1 2>&1 >/dev/null |
        grep -q "$2" || fail "trec_eval $D/$1: no warning '$2'"
}

expect_output $D/out.over -q -m relstring $D/qrels $D/run.over
warns run.over "score '1e401' in line 1 out of range, read as inf"
warns run.over "score '1e400' in line 2 out of range, read as inf"
warns run.over "score '-1e400' in line 3 out of range, read as -inf"

expect_output $D/out.under -q -m relstring $D/qrels $D/run.under
warns run.under "score '2e-400' in line 1 out of range, read as 0"
warns run.under "score '1e-400' in line 2 out of range, read as 0"
warns run.under "score '-1e-400' in line 3 out of range, read as -0"

warns run.comment "score '1e400' in line 2 out of range, read as inf"

expect_output $D/out.subnormal -q -m relstring $D/qrels $D/run.subnormal
"$TREC_EVAL" -q -m relstring $D/qrels $D/run.subnormal 2>&1 >/dev/null |
    grep -q "out of range" && fail "trec_eval $D/run.subnormal: a warning"
exit 0
