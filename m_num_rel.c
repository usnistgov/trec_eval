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

/*     "    Number of relevant documents for topic. \n\
    May be affected by Judged_docs_only and Max_retrieved_per_topic command\n\
    line parameters (as are most measures).\n\
    Summary figure is sum of individual topics, not average.\n",
*/

int 
te_calc_num_rel (const EPI *epi, const REL_INFO *rel_info,
		 const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    eval->values[tm->eval_index].value = (double) rank_rel.num_rel;
    return (1);
}
