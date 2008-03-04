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

/*      "    Precision measured at multiples of R (num_rel).\n\
    This is an attempt to measure topics at the same multiple milestones\n\
    in a retrieval (see explanation of R-prec), in order to determine\n\
    whether methods are precision oriented or recall oriented.  If method A\n\
    dominates method B at the low multiples but performs less well at the\n\
    high multiples then it is precision oriented (compared to B).\n",
*/

int 
te_calc_Rprec_at (const EPI *epi, const REL_INFO *rel_info,
		  const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    FLOAT_PARAMS *cutoff_percents = (FLOAT_PARAMS *) tm->meas_params;
    long *cutoffs;    /* cutoffs expressed in num ret docs instead of percents*/
    long current_cut; /* current index into cutoffs */
    RANK_REL rr;
    long rel_so_far;
    long i;
    double precis, int_precis;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rr))
	return (UNDEF);

    /* translate percentage of rels as given in the measure params, to
       an actual cutoff number of docs. */
    if (NULL == (cutoffs = Malloc (cutoff_percents->num_params, long)))
	return (UNDEF);
    for (i = 0; i < cutoff_percents->num_params; i++)
	cutoffs[i] = (long)(cutoff_percents->param_values[i] * rr.num_rel +0.9);
    precis = (double) rr.num_rel_ret / (double) rr.num_ret;
    int_precis = precis;

    current_cut = cutoff_percents->num_params - 1;
    while (current_cut >= 0 && cutoffs[current_cut] > rr.num_ret) {
	eval->values[tm->eval_index + current_cut].value =
	    (double) rr.num_rel_ret / (double) cutoffs[current_cut];
	current_cut--;
    }

    /* Loop over all retrieved docs in reverse order.  */
    rel_so_far = rr.num_rel_ret;
    for (i = rr.num_ret; i > 0 && rel_so_far > 0; i--) {
	precis = (double) rel_so_far / (double) i;
	if (int_precis < precis)
	    int_precis = precis;
	while (current_cut >= 0 && i == cutoffs[current_cut]) {
	    eval->values[tm->eval_index + current_cut].value = precis;
	    current_cut--;
	}
	if (rr.results_rel_list[i-1] >= epi->relevance_level) {
            rel_so_far--;
	}
    }

    (void) Free (cutoffs);

    return (1);
}
