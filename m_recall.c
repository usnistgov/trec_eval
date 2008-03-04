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

/*      "    Recall at cutoffs\n\
    Recall (relevant retrieved / relevant) measured at various doc level\n\
    cutoffs in the ranking. If the cutoff is larger than the number of docs\n\
    retrieved, then it is assumed nonrelevant docs fill in the rest.\n\
    Recall is a fine single topic measure, but does not average well.\n\
    Cutoffs must be positive without duplicates\n\
    Default param: trec_eval -m recall.5,10,15,20,30,100,200,500,1000\n",
*/

int 
te_calc_recall (const EPI *epi, const REL_INFO *rel_info,
		const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    long *cutoffs = (long *) tm->meas_params->param_values;
    long cutoff_index = 0;
    long i;
    RES_RELS res_rels;
    long rel_so_far = 0;

    if (UNDEF == te_form_res_rels (epi, rel_info, results, &res_rels))
	return (UNDEF);

    for (i = 0; i < res_rels.num_ret; i++) {
	if (i == cutoffs[cutoff_index]) {
	    /* Calculate previous cutoff threshold.
	     Note cutoffs guaranteed to be positive by init_meas */
	    eval->values[tm->eval_index + cutoff_index].value =
		(double) rel_so_far / (double) res_rels.num_rel;
	    if (++cutoff_index == tm->meas_params->num_params)
		break;
	}
	if (res_rels.results_rel_list[i] >= epi->relevance_level)
	    rel_so_far++;
    }
    /* calculate values for those cutoffs not achieved */
    while (cutoff_index < tm->meas_params->num_params) {
	eval->values[tm->eval_index + cutoff_index].value =
	    (double) rel_so_far / (double) res_rels.num_rel;
	cutoff_index++;
    }
    return (1);
}
