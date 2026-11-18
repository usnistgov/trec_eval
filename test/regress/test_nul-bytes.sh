# A judgments or results file that contains a NUL byte is refused with
# exit 2 and a message naming the file. Before, reading stopped at the
# NUL, or the parser ran past the end of the buffer. The files with a NUL
# are written here, so that the fixtures stay text.
. test/regress/lib.sh
D=test/regress/nul-bytes
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

# A NUL on a line of its own, and a NUL inside the docno field.
printf '1 0 d1 1\n\000\n1 0 d2 1\n' > $T/qrels.nul-line
printf '1 0 d1\000x 1\n' > $T/qrels.nul-field
printf '1 A 0 d1 1\n\000\n1 A 0 d2 1\n' > $T/prefs.nul-line
printf '1 A 0 d1\000x 1\n' > $T/prefs.nul-field
printf '1 Q0 d1 1 3 r\n\000\n1 Q0 d3 3 1 r\n' > $T/run.nul-line
printf '1 Q0 d1\000x 1 3 r\n' > $T/run.nul-field

# check JUDGMENTS RUN ARGS...
check() {
    judg=$1 run=$2
    shift 2
    expect_status 2 "$@" $judg $run
    "$TREC_EVAL" "$@" $judg $run 2>&1 >/dev/null |
        grep -q "NUL character in .* file '.*nul-.*'" ||
        fail "trec_eval $* $judg $run: no message naming the file"
}

for v in nul-line nul-field; do
    check $T/qrels.$v $D/run -m num_rel
    check $T/qrels.$v $D/run -R qrels_jg -m map_avgjg
    check $T/qrels.$v $D/run -R qrels_prefs -m prefs_num_prefs_poss
    check $T/prefs.$v $D/run -R prefs -m prefs_num_prefs_poss
    check $D/qrels $T/run.$v -m num_ret
done

# Results read from standard input are not checked, and still evaluate.
expect_output test/out.test test/qrels.test - < test/results.test
exit 0
