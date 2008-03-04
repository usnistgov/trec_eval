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

/*      "    Precision measured at multiples of R(num_rel) averged over users.\n\
    This is an attempt to measure topics at the same multiple milestones\n\
    in a retrieval (see explanation of R-prec), in order to determine\n\
    whether methods are precision oriented or recall oriented.  If method A\n\
    dominates method B at the low multiples but performs less well at the\n\
    high multiples then it is precision oriented (compared to B).\n\
    If there is more than one judgment group (set of evalutation judgments\n\
    of a user), then the measure is averaged over those jgs.\n",
*/

int 
te_calc_Rprec_mult_avgjg (const EPI *epi, const REL_INFO *rel_info,
			  const RESULTS *results, const TREC_MEAS *tm,
			  TREC_EVAL *eval)
{
    double *cutoff_percents = (double *) tm->meas_params->param_values;
    long *cutoffs;    /* cutoffs expressed in num ret docs instead of percents*/
    long current_cut; /* current index into cutoffs */
    RES_RELS_JG rr;
    long rel_so_far;
    long i, jg;
    double precis, int_precis;

    if (UNDEF == te_form_res_rels_jg (epi, rel_info, results, &rr))
	return (UNDEF);

    /* Need to translate percentage of rels as given in the measure params, to
       an actual cutoff number of docs. */
    if (NULL == (cutoffs = Malloc (tm->meas_params->num_params, long)))
	return (UNDEF);

    for (jg = 0; jg < rr.num_jgs; jg++) {
	for (i = 0; i < tm->meas_params->num_params; i++)
	    cutoffs[i] = (long)(cutoff_percents[i]
				* rr.jgs[jg].num_rel +0.9);
	precis = (double) rr.jgs[jg].num_rel_ret / (double) rr.jgs[jg].num_ret;
	int_precis = precis;
	
	current_cut = tm->meas_params->num_params - 1;
	while (current_cut >= 0 && cutoffs[current_cut] > rr.jgs[jg].num_ret) {
	    eval->values[tm->eval_index + current_cut].value +=
		(double) rr.jgs[jg].num_rel_ret / (double) cutoffs[current_cut];
	    current_cut--;
	}
	
	/* Loop over all retrieved docs in reverse order.  */
	rel_so_far = rr.jgs[jg].num_rel_ret;
	for (i = rr.jgs[jg].num_ret; i > 0 && rel_so_far > 0; i--) {
	    precis = (double) rel_so_far / (double) i;
	    if (int_precis < precis)
		int_precis = precis;
	    while (current_cut >= 0 && i == cutoffs[current_cut]) {
		eval->values[tm->eval_index + current_cut].value += precis;
		current_cut--;
	    }
	    if (rr.jgs[jg].results_rel_list[i-1] >= epi->relevance_level) {
		rel_so_far--;
	    }
	}
    }

    if (rr.num_jgs > 1) {
	for (current_cut = 0;
	     current_cut < tm->meas_params->num_params;
	     current_cut++)
	    eval->values[tm->eval_index + current_cut].value /= rr.num_jgs;
    }
	
    Free (cutoffs);

    return (1);
}
