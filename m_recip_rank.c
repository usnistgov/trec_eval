/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
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
    RES_RELS res_rels;
    long i;

    if (UNDEF == te_form_res_rels (epi, rel_info, results, &res_rels))
	return (UNDEF);

    for (i = 0; i < res_rels.num_ret; i++) {
	if (res_rels.results_rel_list[i] >= epi->relevance_level)
	    break;
    }
    if (i <  res_rels.num_ret)
	eval->values[tm->eval_index].value =
	    (double) 1.0 / (double) (i+1);
    return (1);
}
