#ifdef RCSID
static char rcsid[] = "$Header: /home/smart/release/src/libevaluate/trvec_trec_eval.c,v 11.0 1992/07/21 18:20:35 chrisb Exp chrisb $";
#endif

/* Copyright (c) 2008 Chris Buckley
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "trec_format.h"

/* Utility malloc procedures for handling a malloc'd reusable object list that
   may need to be resized.
   Procedures take a current pointer, the current bound on the number of
   objects in the list, the needed number of objects on the list, and the
   size of an object.
   The procedures return the current pointer if the needed number is less
   than the current bound, otherwise they expand the space (either malloc
   or realloc depending on the procedure), resetting the current bound.
   It's an error if current_bound is negative, NULL is returned.
   Procedures assume that if current bound is 0, the space has not been
   allocated at all yet.
*/   

void *
te_chk_and_malloc (void *ptr, long *current_bound,
		const long needed, const size_t size)
{
    if (*current_bound < 0)
	return (NULL);
    if (needed <= *current_bound)
	return (ptr);
    if (*current_bound > 0)
	Free (ptr);
    *current_bound += needed;
    return ((void *) malloc (*current_bound * size));
}

void *
te_chk_and_realloc (void *ptr, long *current_bound,
		 const long needed, const int size)
{
    if (*current_bound < 0)
	return (NULL);
    if (needed <= *current_bound)
	return (ptr);
    if (*current_bound == 0) {
	*current_bound += needed;
	return ((void *) malloc (*current_bound * size));
    }
    *current_bound += needed;
    return ((void *) realloc (ptr, *current_bound * size));
}
