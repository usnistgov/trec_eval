/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/
#ifndef COMMONH
#define COMMONH

#include <stdio.h>

#ifndef FALSE
#define FALSE   0
#endif
#ifndef TRUE
#define TRUE    1
#endif

#define UNDEF   -1

#define MAX(A,B)  ((A) > (B) ? (A) : (B))
#define MIN(A,B)  ((A) > (B) ? (B) : (A))

#ifndef MAXLONG
#define MAXLONG 2147483647L     /* largest long int. no. */
#endif

/*
 * Some useful macros for making malloc et al easier to use.
 * Macros handle the casting and the like that's needed.
 * Malloc and Realloc return NULL, without allocating, if n * sizeof(type)
 * does not fit in a size_t. They evaluate n twice.
 */
#define TE_SIZE_FITS(n,type) ((size_t) (n) <= ((size_t) -1) / sizeof(type))
#define Malloc(n,type) (type *) (TE_SIZE_FITS(n,type) ? \
                                 malloc((size_t) (n) * sizeof(type)) : NULL)
#define Realloc(loc,n,type) (type *) (TE_SIZE_FITS(n,type) ? \
                                      realloc((char *)(loc), \
                                              (size_t) (n) * sizeof(type)) : \
                                      NULL)
#define Free(loc) (void) free( (char *)(loc) )

#endif                          /* COMMONH */
