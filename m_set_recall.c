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

/*    "    Set Recall: num_relevant_retrieved / num_relevant \n\
    Recall over all docs retrieved for a topic.\n\
    Was known as exact_recall in earlier versions of trec_eval",
*/

int 
te_calc_set_recall (const EPI *epi, const REL_INFO *rel_info,
		    const RESULTS *results, const TREC_MEAS *tm,
		    TREC_EVAL *eval)
{
    RANK_REL rank_rel;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    eval->values[tm->eval_index].value =
	(double) rank_rel.num_rel_ret /
	(double) rank_rel.num_rel;

    return (1);
}
