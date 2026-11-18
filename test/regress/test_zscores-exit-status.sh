# A zscores file (-Z) that cannot be read, or has a malformed line, stops
# trec_eval with exit status 2 and "Quit in file", as for the judgments
# and results files. Before, main returned UNDEF and the status was 255.
. test/regress/lib.sh
d=test/regress/zscores-exit-status

for z in $d/zscores.malformed $d/no-such-file; do
    expect_status 2 -q -Z $z -m map $d/qrels $d/results
    expect_output $d/out.empty -q -Z $z -m map $d/qrels $d/results
    "$TREC_EVAL" -q -Z $z -m map $d/qrels $d/results 2>&1 >/dev/null |
        grep -q "^trec_eval: Quit in file '$z'$" ||
        fail "-Z $z: no message naming the file"
done

expect_status 0 -q -Z $d/zscores -m map $d/qrels $d/results
expect_output $d/out.valid -q -Z $d/zscores -m map $d/qrels $d/results
