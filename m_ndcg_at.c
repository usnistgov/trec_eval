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

double log2(double x);

/* From implementation by Ian Soboroff */
/* ndcg at document cutoffs */
/*     "    Normalized Discounted Cumulative Gain at cutoffs.\n\
    Compute a traditional nDCG measure according to Jarvelin and\n\
    Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002) at cutoffs.\n\
    Gain values are the relevance values in the qrels file.  For now, if you\n\
    want different gains, change the qrels file appropriately.\n\
    Cutoffs must be positive without duplicates\n\
    Default usage: trec_eval -m ndcg_at.5,10,15,20,30,100,200,500,1000\n"
*/

int 
te_calc_ndcg_at (const EPI *epi, const REL_INFO *rel_info,
		 const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    LONG_PARAMS *cutoffs = (LONG_PARAMS *) tm->meas_params;
    long cutoff_index = 0;
    RANK_REL rank_rel;
    double gain, sum;
    double ideal_dcg;          /* ideal discounted cumulative gain */
    long cur_lvl, lvl_count;
    long i;
   
    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    sum = 0.0;
    for (i = 0; i < rank_rel.num_ret; i++) {
        if (i == cutoffs->param_values[cutoff_index]) {
            /* Calculate previous cutoff threshold.
             Note i guaranteed to be positive by init_meas */
            eval->values[tm->eval_index + cutoff_index].value = sum;
            if (++cutoff_index == cutoffs->num_params)
                break;
	    if (epi->debug_level > 0) 
		printf("ndcg_at: cutoff %ld dcg %6.4f\n", i, sum);
        }
	gain = rank_rel.results_rel_list[i];
	if (gain > 0) {
	    if (i > 0)
		sum += gain / log2((double) (i+1));
	    else
		sum += gain;
	    if (epi->debug_level > 1) 
		printf("ndcg_at:%ld %3.1f %6.4f\n", i, gain, sum);
	}
    }
    /* calculate values for those cutoffs not achieved */
    while (cutoff_index < cutoffs->num_params) {
	eval->values[tm->eval_index + cutoff_index].value = sum;
	if (epi->debug_level > 0) 
	    printf("ndcg_at: cutoff %ld dcg %6.4f\n",
		   cutoffs->param_values[cutoff_index], sum);
        cutoff_index++;
    }
    /* Calculate ideal discounted cumulative gain for this topic, and 
     normalize previous sum by it */
    cutoff_index = 0;
    cur_lvl = rank_rel.num_rel_levels - 1;
    lvl_count = 0;
    ideal_dcg = 0.0;
    for (i = 0; 1; i++) {
	lvl_count++;
	while (lvl_count > rank_rel.rel_levels[cur_lvl]) {
	    cur_lvl--;
	    if (cur_lvl == 0)
		break;
	    lvl_count = 1;
	}
	if (cur_lvl == 0)
	    break;

        if (i == cutoffs->param_values[cutoff_index]) {
            /* Calculate previous cutoff threshold.
             Note i guaranteed to be positive by init_meas */
	    if (ideal_dcg > 0.0) 
		eval->values[tm->eval_index + cutoff_index].value /= ideal_dcg;
	    if (epi->debug_level > 0)
		printf("ndcg_at: cutoff %ld idcg %6.4f\n", i, ideal_dcg);
            if (++cutoff_index == cutoffs->num_params)
                break;
        }
	gain = cur_lvl;
	if (i == 0)
	    ideal_dcg += gain;
	else
	    ideal_dcg += gain / (double) log2((double)(i + 1));
	if (epi->debug_level > 0) 
	    printf("ndcg_at:%ld %ld %3.1f %6.4f\n", i, cur_lvl, gain,ideal_dcg);
    }

    /* calculate values for those cutoffs not achieved */
    while (cutoff_index < cutoffs->num_params) {
	if (ideal_dcg > 0.0) 
	    eval->values[tm->eval_index + cutoff_index].value /= ideal_dcg;
	if (epi->debug_level > 0)
	    printf("ndcg_at: cutoff %ld idcg %6.4f\n",
		   cutoffs->param_values[cutoff_index], ideal_dcg);
        cutoff_index++;
    }

    return (1);
}
