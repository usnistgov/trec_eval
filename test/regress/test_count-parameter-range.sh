# A float parameter whose count of documents does not fit in a long is
# clamped: it gives the same values, with exit 0, as a large value that
# fits. Before, the conversion to long was undefined (the casts in
# relstring, Rprec_mult and Rprec_mult_avgjg) or unspecified (lround in
# iprec_at_recall and 11pt_avg). With a build made with
# -fsanitize=undefined, no runtime error may be reported either.
# relstring.1e300 prints the whole ranking of d1, d2, d3: '102'.
. test/regress/lib.sh
D=test/regress/count-parameter-range

# same MEASURE HUGE LARGE [ARGS...]: MEASURE.HUGE exits 0 without a
# runtime error and prints the values that MEASURE.LARGE prints.
same() {
    m=$1 huge=$2 large=$3
    shift 3
    expect_status 0 -q "$@" -m $m.$huge $D/qrels $D/run
    "$TREC_EVAL" -q "$@" -m $m.$huge $D/qrels $D/run 2>&1 >/dev/null |
        grep -q "runtime error" && fail "trec_eval -m $m.$huge: runtime error"
    a=$("$TREC_EVAL" -q "$@" -m $m.$huge $D/qrels $D/run | cut -f2-)
    b=$("$TREC_EVAL" -q "$@" -m $m.$large $D/qrels $D/run | cut -f2-)
    [ "$a" = "$b" ] || fail "trec_eval -m $m.$huge differs from $m.$large"
}

# Recall levels outside [0, 1] and multiples of R that are not positive
# are refused, so only relstring and large multiples of R remain.
for m in relstring Rprec_mult; do
    same $m 1e300 1000000
done
same relstring -1e300 -1000000
same Rprec_mult_avgjg 1e300 1000000 -R qrels_jg

expect_output $D/out.relstring -q -m relstring.1e300 $D/qrels $D/run
exit 0
