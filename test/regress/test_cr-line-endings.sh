# Lines may end with LF or CRLF. A CR that is not followed by LF, as in a
# file with CR-only line endings, is refused with a message naming the
# line and the file. Before, a CR-only results file was read as one line
# and evaluated without a message. The CRLF and CR files are written here
# from the LF fixtures, so that the fixtures stay text.
. test/regress/lib.sh
D=test/regress/cr-line-endings
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

for f in qrels prefs results zscores; do
    awk '{ printf "%s\r\n", $0 }' $D/$f > $T/$f.crlf
    tr '\n' '\r' < $D/$f > $T/$f.cr
done
printf '1 0 d1 1\n1 0 d2 0\r1 0 d3 1\n' > $T/qrels.mixed

# CRLF files give the same values as the LF files.
for e in "" .crlf; do
    q=$D/qrels r=$D/results
    [ -n "$e" ] && q=$T/qrels$e r=$T/results$e
    expect_output $D/out.qrels -q -m num_ret -m map $q $r
    expect_output $D/out.qrels -q -m num_ret -m map $q - < $r
    expect_output $D/out.qrels_jg -q -R qrels_jg -m map_avgjg $q $r
    expect_output $D/out.qrels_prefs -q -R qrels_prefs \
        -m prefs_num_prefs_poss $q $r
done
expect_output $D/out.prefs -q -R prefs -m prefs_num_prefs_poss \
    $T/prefs.crlf $D/results
expect_output $D/out.zscores -q -Z $T/zscores.crlf -m map $D/qrels $D/results

# refused LINE FILE ARGS...: exit 2 and a message naming LINE and FILE.
refused() {
    line=$1 file=$2
    shift 2
    expect_status 2 "$@"
    "$TREC_EVAL" "$@" 2>&1 >/dev/null |
        grep -q "CR character without LF in line $line of .* file '$file'" ||
        fail "trec_eval $*: no message naming line $line of $file"
}
refused 1 $T/results.cr -m num_ret $D/qrels $T/results.cr
refused 1 $T/qrels.cr -m num_rel $T/qrels.cr $D/results
refused 1 $T/qrels.cr -R qrels_jg -m map_avgjg $T/qrels.cr $D/results
refused 1 $T/qrels.cr -R qrels_prefs -m prefs_num_prefs_poss \
    $T/qrels.cr $D/results
refused 1 $T/prefs.cr -R prefs -m prefs_num_prefs_poss \
    $T/prefs.cr $D/results
refused 2 $T/qrels.mixed -m num_rel $T/qrels.mixed $D/results
expect_status 2 -m num_ret $D/qrels - < $T/results.cr
"$TREC_EVAL" -m num_ret $D/qrels - < $T/results.cr 2>&1 >/dev/null |
    grep -q "CR character without LF in line 1 of results file '-'" ||
    fail "results on stdin: no message naming the line"
# A zscores file gets the same message.
"$TREC_EVAL" -Z $T/zscores.cr -m map $D/qrels $D/results 2>&1 >/dev/null |
    grep -q "CR character without LF in line 1 of zscores file" ||
    fail "-Z $T/zscores.cr: no message naming the line"
