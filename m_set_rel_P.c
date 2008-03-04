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

/*Relative Set Precision:  P / (Max possible P for this size set) \n \
    where P is num_relevant_retrieved / num_retrieved\n\
    Relative precision over all docs retrieved for a topic.\n\
    Was known as exact_relative_prec in earlier versions of trec_eval\n\
    Note:   trec_eval -m relative_P.50 ...\n\
    is different from \n\
    trec_eval -M 50 -m set_relative_P ...\n */

int 
te_calc_set_relative_P (const EPI *epi, const REL_INFO *rel_info,
			const RESULTS *results, const TREC_MEAS *tm,
			TREC_EVAL *eval)
{
    RANK_REL rank_rel;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    if (rank_rel.num_ret) 
	eval->values[tm->eval_index].value =
	    (double) rank_rel.num_rel_ret /
	    (double) MIN (rank_rel.num_ret, rank_rel.num_rel);
    
    return (1);
}
