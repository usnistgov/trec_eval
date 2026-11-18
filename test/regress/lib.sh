# Helpers for the scripts in test/regress. Each script is run from the
# repository root as `sh test/regress/test_<name>.sh ./trec_eval`.
TREC_EVAL=${1:-./trec_eval}

fail() {
    echo "FAIL $0: $*" >&2
    exit 1
}

# expect_output FILE ARGS...: standard output must equal FILE.
expect_output() {
    file=$1
    shift
    "$TREC_EVAL" "$@" 2>/dev/null | diff - "$file" >/dev/null ||
        fail "trec_eval $* differs from $file"
}

# expect_status STATUS ARGS...: the exit status must equal STATUS.
expect_status() {
    want=$1
    shift
    "$TREC_EVAL" "$@" >/dev/null 2>&1
    got=$?
    [ "$got" = "$want" ] || fail "trec_eval $*: exit $got, expected $want"
}
