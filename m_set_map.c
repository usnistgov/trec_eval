#ifdef RCSID
static char rcsid[] = "$Header: /home/smart/release/src/libevaluate/trvec_trec_eval.c,v 11.0 1992/07/21 18:20:35 chrisb Exp chrisb $";
#endif

/* Copyright (c) 2008
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "trec_format.h"

/*  Set map: num_relevant_retrieved**2 / (num_retrieved*num_rel)
    Unranked set map, where the precision due to all relevant retrieved docs
    is the set precision, and the precision due to all relevant not-retrieved
    docs is set to 0.
    Was known as exact_unranked_avg_prec in earlier versions of trec_eval
    Another way of loooking at this is  Recall * Precision on the set of
    docs retrieved for a topic. 
*/

int 
te_calc_set_map (const EPI *epi, const REL_INFO *rel_info,
		 const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    if (rank_rel.num_ret)
	eval->values[tm->eval_index].value =
	    ((double) rank_rel.num_rel_ret *
	     (double) rank_rel.num_rel_ret)      /
	    ((double) rank_rel.num_ret *
	     (double) rank_rel.num_rel);

    return (1);
}
