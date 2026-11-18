# Two cutoffs of iprec_at_recall, Rprec_mult or Rprec_mult_avgjg that give
# the same printed name are refused with exit status 2. Names show a
# cutoff to two decimals, so 0.999 and 1.001 both gave Rprec_mult_1.00,
# and the two rows could not be told apart. Cutoffs whose names differ
# give the same output as before.
. test/regress/lib.sh
D=test/regress/cutoff-name-collisions
Q=test/qrels.test
R=test/results.test

# refused TEXT ARGS...: exit status 2, and TEXT on standard error.
refused() {
    text=$1
    shift
    expect_status 2 "$@"
    "$TREC_EVAL" "$@" 2>&1 >/dev/null | grep -F -q "$text" ||
        fail "trec_eval $*: no message containing: $text"
}

refused "cutoffs 0.999 and 1.001 both give the name 'Rprec_mult_1.00'" \
    -m Rprec_mult.0.999,1.001 $Q $R
refused "cutoffs 0.101 and 0.104 both give the name 'iprec_at_recall_0.10'" \
    -m iprec_at_recall.0.101,0.104 $Q $R
refused "cutoffs 0.999 and 1.001 both give the name 'Rprec_mult_avgjg_1.00'" \
    -R qrels_jg -m Rprec_mult_avgjg.0.999,1.001 test/qrels.rel_level $R

expect_output $D/out.distinct -m Rprec_mult.0.995,1.004 \
    -m iprec_at_recall.0.1,0.106 $Q $R
# Large cutoffs whose names were cut to the same eight characters now
# have distinct names, and are accepted.
expect_status 0 -m Rprec_mult.100000.01,100000.02 $Q $R
expect_status 0 -m Rprec_mult.10000000.5,50000000,100000000 $Q $R
exit 0
