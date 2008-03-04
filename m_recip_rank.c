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

/*     "    Reciprocal Rank of the first relevant retrieved doc.\n\
    Measure is most useful for tasks in which there is only one relevant\n\
    doc, or the user only wants one relevant doc.\n",
*/

int 
te_calc_recip_rank (const EPI *epi, const REL_INFO *rel_info,
		    const RESULTS *results, const TREC_MEAS *tm,
		    TREC_EVAL *eval)
{
    RANK_REL rank_rel;
    long i;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    for (i = 0; i < rank_rel.num_ret; i++) {
	if (rank_rel.results_rel_list[i] >= epi->relevance_level)
	    break;
    }
    if (i <  rank_rel.num_ret)
	eval->values[tm->eval_index].value =
	    (double) 1.0 / (double) (i+1);
    return (1);
}
