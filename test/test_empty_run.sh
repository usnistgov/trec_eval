#!/bin/sh
# Regression tests for run files with no data lines (issue #99).
set -eu

BIN="${1:-./trec_eval}"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_exit2() {
    desc="$1"; shift
    rc=0
    "$@" >"$TMPDIR/out" 2>"$TMPDIR/err" || rc=$?
    [ "$rc" -eq 2 ] || fail "$desc: expected exit 2, got $rc (stdout=$(cat "$TMPDIR/out") stderr=$(cat "$TMPDIR/err"))"
    grep -q "Quit in file" "$TMPDIR/err" || fail "$desc: expected 'Quit in file' on stderr, got: $(cat "$TMPDIR/err")"
}

assert_succeeds() {
    desc="$1"; shift
    if ! "$@" >"$TMPDIR/out" 2>"$TMPDIR/err"; then
        fail "$desc: expected exit 0, got $? (stderr=$(cat "$TMPDIR/err"))"
    fi
}

printf '1 0 a 1\n1 0 b 0\n' > "$TMPDIR/one.qrels"
printf '\n\n' > "$TMPDIR/blank.run"
printf '   \n\t\r\n' > "$TMPDIR/spaces.run"
printf '# a comment\n' > "$TMPDIR/comment.run"
printf '# a comment' > "$TMPDIR/comment_nonl.run"
printf '# first comment\n# second comment\n' > "$TMPDIR/two_comments.run"
printf '  # indented comment\n' > "$TMPDIR/indented_comment.run"
printf '\n# comment\n  \n' > "$TMPDIR/mixed_empty.run"
: > "$TMPDIR/empty.run"

# Issue #99 reproduction cases
assert_exit2 "blank.run" "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/blank.run"
assert_exit2 "comment.run" "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/comment.run"
assert_exit2 "empty stdin" sh -c '"$1" "$2" - < /dev/null' _ "$BIN" "$TMPDIR/one.qrels"
assert_exit2 "empty.run" "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/empty.run"

# Additional no-data-line edge cases
assert_exit2 "spaces.run" "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/spaces.run"
assert_exit2 "comment without trailing newline" "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/comment_nonl.run"
assert_exit2 "two comments" "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/two_comments.run"
assert_exit2 "indented comment" "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/indented_comment.run"
assert_exit2 "mixed blank and comment" "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/mixed_empty.run"
assert_exit2 "stdin blank" sh -c 'printf "\n\n" | "$1" "$2" -' _ "$BIN" "$TMPDIR/one.qrels"
assert_exit2 "stdin comment" sh -c 'printf "# a comment\n" | "$1" "$2" -' _ "$BIN" "$TMPDIR/one.qrels"
assert_exit2 "-c blank.run" "$BIN" -c "$TMPDIR/one.qrels" "$TMPDIR/blank.run"
assert_exit2 "-c empty stdin" sh -c '"$1" -c "$2" - < /dev/null' _ "$BIN" "$TMPDIR/one.qrels"

# Valid runs with comments and blank lines (file and stdin) still succeed
printf '# header\n\n1 Q0 a 1 1.0 r\n\n# footer\n' > "$TMPDIR/valid.run"
assert_succeeds "valid run with comments and blanks" "$BIN" -m num_ret "$TMPDIR/one.qrels" "$TMPDIR/valid.run"
grep -q "num_ret.*all.*1" "$TMPDIR/out" || fail "valid run expected num_ret=1, got: $(cat "$TMPDIR/out")"
assert_succeeds "valid run via stdin" sh -c '"$1" -m num_ret "$2" - < "$3"' _ "$BIN" "$TMPDIR/one.qrels" "$TMPDIR/valid.run"
grep -q "num_ret.*all.*1" "$TMPDIR/out" || fail "valid stdin run expected num_ret=1, got: $(cat "$TMPDIR/out")"
