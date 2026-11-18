# The message for a judgments or results file that contains a NUL byte
# ends with the number of the line with the first NUL, counting every
# line of the file. Before, it named only the file. The exit status
# stays 2.
. test/regress/lib.sh
D=test/regress/nul-bytes
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

printf '1 0 d1 1\n\000\n1 0 d2 1\n' > $T/qrels.2
printf '1 0 d1\000x 1\n' > $T/qrels.1
printf '# c\n1 0 d1 1\n1 0 d2\000 1\n' > $T/qrels.3
printf '1 A 0 d1 1\n\000\n1 A 0 d2 1\n' > $T/prefs.2
printf '1 Q0 d1 1 3 r\n\000\n1 Q0 d3 3 1 r\n' > $T/run.2
printf '1 Q0 d1\000x 1 3 r\n' > $T/run.1

# check TEXT ARGS...: exit status 2, and TEXT on standard error.
check() {
    text=$1
    shift
    expect_status 2 "$@"
    "$TREC_EVAL" "$@" 2>&1 >/dev/null | grep -F -q "$text" ||
        fail "trec_eval $*: no message containing: $text"
}

for n in 1 2 3; do
    check "get_qrels: NUL character in qrels file '$T/qrels.$n' at line $n" \
        -m num_rel $T/qrels.$n $D/run
    check "get_qrels_jg: NUL character in qrels file '$T/qrels.$n' at line $n" \
        -R qrels_jg -m map_avgjg $T/qrels.$n $D/run
    check "get_qrels_prefs: NUL character in qrels file '$T/qrels.$n' at line $n" \
        -R qrels_prefs -m prefs_num_prefs_poss $T/qrels.$n $D/run
done
check "get_prefs: NUL character in prefs file '$T/prefs.2' at line 2" \
    -R prefs -m prefs_num_prefs_poss $T/prefs.2 $D/run
for n in 1 2; do
    check "get_results: NUL character in results file '$T/run.$n' at line $n" \
        -m num_ret $D/qrels $T/run.$n
done
exit 0
