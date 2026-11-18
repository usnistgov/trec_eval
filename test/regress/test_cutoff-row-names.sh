# A row name shows a floating-point cutoff in full, with two decimals.
# The name used to keep only the first eight characters of the number,
# so Rprec_mult.100000000 printed Rprec_mult_10000000, and 100000.01
# and 100000.02 both printed Rprec_mult_100000.0. Names whose number has
# eight characters or fewer, such as Rprec_mult_0.20 and
# Rprec_mult_99999.99, are unchanged; the values are unchanged too.
. test/regress/lib.sh
D=test/regress/cutoff-row-names

expect_output $D/out.Rprec_mult \
    -m Rprec_mult.0.2,99999.99,100000.01,100000.02,100000000 \
    test/qrels.test test/results.test
expect_output $D/out.avgjg -R qrels_jg -m Rprec_mult_avgjg.0.2,100000000 \
    test/qrels.rel_level test/results.test
exit 0
