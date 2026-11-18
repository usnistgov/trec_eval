# A UTF-8 byte-order mark at the start of an input file is skipped, so a
# file gives the same values with or without one. Before, the mark became
# part of the first topic id. The files with a mark are written here from
# the fixtures, so that the fixtures stay text.
. test/regress/lib.sh
D=test/regress/utf8-bom
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

for f in qrels prefs results zscores; do
    { printf '\357\273\277'; cat $D/$f; } > $T/$f
done
{ printf '\357\273\277# comment\n'; cat $D/qrels; } > $T/qrels.comment

m="-c -q -m num_q -m num_ret -m num_rel -m map"
expect_output $D/out.qrels $m $T/qrels $D/results
expect_output $D/out.qrels $m $T/qrels.comment $D/results
expect_output $D/out.qrels $m $D/qrels $T/results
expect_output $D/out.qrels $m $D/qrels - < $T/results
expect_output $D/out.qrels_jg -q -R qrels_jg -m map_avgjg \
    $T/qrels $D/results
expect_output $D/out.qrels_prefs -q -R qrels_prefs -m prefs_num_prefs_poss \
    $T/qrels $D/results
expect_output $D/out.prefs -q -R prefs -m prefs_num_prefs_poss \
    $T/prefs $D/results
expect_output $D/out.zscores -q -Z $T/zscores -m num_ret -m num_rel \
    $D/qrels $D/results
