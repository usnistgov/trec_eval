# Malloc and Realloc (common.h) return NULL when n * sizeof(type) does
# not fit in a size_t. They used to cast the byte count to unsigned, so
# a count of 4 GiB or more was cut to its low 32 bits. Judgments files of
# 2 GiB or more are impractical here, so this script checks the macros
# with a small program built against common.h.
. test/regress/lib.sh
D=test/regress/large-file-sizes
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

cc=
for c in "${CC:-}" cc gcc clang; do
    if [ -n "$c" ] && command -v "$c" >/dev/null 2>&1; then
        cc=$c
        break
    fi
done
if [ -z "$cc" ]; then
    echo "SKIP $0: no C compiler found" >&2
    exit 0
fi
"$cc" -I. -o "$T/alloc" $D/alloc.c || fail "cannot build $D/alloc.c"
"$T/alloc" || fail "Malloc or Realloc allocated a size that does not fit"
exit 0
