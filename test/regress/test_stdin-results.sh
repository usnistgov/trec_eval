# Results read from standard input (-) are checked for a NUL byte as a
# results file is: exit 2 and a message. Before, a NUL dropped the rest
# of its line, newline included, and the line was joined to the next.
# Input of any size is read whole: the reader's buffer starts at 4096
# bytes, and inputs near that size give the same output as the file.
. test/regress/lib.sh
D=test/regress/stdin-results
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

printf '1 Q0 d1 1 3 r\n1 Q0 d2\000x 2 2 r\n1 Q0 d3 3 1 r\n' > $T/run.nul-field
printf '1 Q0 d1 1 3 r\n\000\n1 Q0 d3 3 1 r\n' > $T/run.nul-line
for v in nul-field nul-line; do
    expect_status 2 -m num_ret $D/qrels - < $T/run.$v
    "$TREC_EVAL" -m num_ret $D/qrels - < $T/run.$v 2>&1 >/dev/null |
        grep -q "NUL character in results file '-'" ||
        fail "trec_eval $D/qrels - < $T/run.$v: no message"
done

expect_output test/out.test test/qrels.test - < test/results.test

# Inputs of n bytes: 40 results for topic 301, then a comment padded to
# n bytes with no final newline.
grep '^301 ' test/qrels.test > $T/qrels.301
head -n 40 test/results.test > $T/head
for n in 4093 4094 4095 8190; do
    pad=$(( n - $(wc -c < $T/head) - 1 ))
    { cat $T/head; printf "#%${pad}s" ''; } > $T/run.$n
    [ "$(wc -c < $T/run.$n)" -eq $n ] || fail "$T/run.$n is not $n bytes"
    "$TREC_EVAL" -q $T/qrels.301 $T/run.$n > $T/out.$n ||
        fail "trec_eval $T/qrels.301 $T/run.$n failed"
    expect_output $T/out.$n -q $T/qrels.301 - < $T/run.$n
done
exit 0
