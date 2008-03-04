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

/*  Set Precision: num_relevant_retrieved / num_retrieved \n	\
    Precision over all docs retrieved for a topic.\n\
    Note:   trec_eval -m P.50 ...\n\
    is different from \n\
            trec_eval -M 50 -m set_P ...\n, */
/* Was known as exact_prec in earlier versions of trec_eval */

int 
te_calc_set_P (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	    const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    if (rank_rel.num_ret) 
	eval->values[tm->eval_index].value =
	    (double) rank_rel.num_rel_ret /
	    (double) rank_rel.num_ret;
    return (1);
}
