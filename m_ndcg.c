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

/*      "    Normalized Discounted Cumulative Gain\n\
    Compute a traditional nDCG measure according to Jarvelin and\n\
    Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002).\n\
    Gain values are the relevance values in the qrels file.  For now, if you\n\
    want different gains, change the qrels file appropriately, or use\n\
    ndcg_p.  Will be replaced by ndcg_p? ... \n",
*/

int 
te_calc_ndcg (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	      const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;
    double gain, sum;
    double ideal_dcg;          /* ideal discounted cumulative gain */
    long cur_lvl, lvl_count;
    long i;
   
    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    sum = 0.0;
    for (i = 0; i < rank_rel.num_ret; i++) {
	gain = rank_rel.results_rel_list[i];
	if (gain > 0) {
	    if (i > 0)
		sum += gain / log2((double) (i+1));
	    else
		sum += gain;
	    if (epi->debug_level > 0) 
		printf("ndcg:%ld %3.1f %6.4f\n", i, gain, sum);
	}
    }
    /* Calculate ideal discounted cumulative gain for this topic */
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
	gain = cur_lvl;
	if (i == 0)
	    ideal_dcg += gain;
	else
	    ideal_dcg += gain / (float) log2((double)(i + 1));
	if (epi->debug_level > 0) 
	    printf("ndcg:%ld %ld %3.1f %6.4f\n", i, cur_lvl, gain, ideal_dcg);
    }

    /* Compare sum to ideal NDCG */
    if (ideal_dcg > 0.0) {
        eval->values[tm->eval_index].value =
	    sum / ideal_dcg;
    }
    return (1);
}
