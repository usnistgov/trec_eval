/* Checks that Malloc and Realloc in common.h return NULL when
   n * sizeof(type) does not fit in a size_t. test_large-file-sizes.sh
   builds and runs it; the exit status is 0 if every check holds. */
#include <stdio.h>
#include <stdlib.h>
#include "common.h"

static int failed = 0;

static void check_null(void *ptr, const char *call)
{
    if (ptr != NULL) {
        printf("%s did not return NULL\n", call);
        failed = 1;
        Free(ptr);
    }
}

int main(void)
{
    /* half * sizeof(short) is one more than the largest size_t */
    size_t half = ((size_t) -1) / 2 + 1;
    char *ptr;

    check_null(Malloc(half, short), "Malloc(SIZE_MAX / 2 + 1, short)");
    check_null(Malloc(-1, long), "Malloc(-1, long)");
    check_null(Realloc(NULL, half, short),
               "Realloc(NULL, SIZE_MAX / 2 + 1, short)");

    /* Sizes that fit are still allocated */
    if (NULL == (ptr = Malloc(16, char)) ||
        NULL == (ptr = Realloc(ptr, 32, char))) {
        printf("Malloc(16, char) or Realloc(ptr, 32, char) returned NULL\n");
        return 1;
    }
    Free(ptr);
    return failed;
}
