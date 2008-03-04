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

/*      "    Interpolated Precision at recall cutoffs.\n\
    This is the data shown in the standard Recall-Precision graph.\n\
    The standard cutoffs and interpolation are needed to average data over\n\
    multiple topics; otherwise, how is a topic with 5 relevant docs averaged\n\
    with a topic with 3 relevant docs for graphing purposes?  The Precision \n\
    interpolation used here is\n\
      Int_Prec (rankX) == MAX (Prec (rankY)) for all Y >= X.\n\
    Default usage: trec_eval -m ircl_prn.0,.1,.2,.3,.4,.5,.6,.7,.8,.9,1. ...\n\
    Name should really be changed (how many abbreviations for 'precision'\n\
    should one program use?), but kept for backward compatibility\n",
*/

int 
te_calc_ircl_prn (const EPI *epi, const REL_INFO *rel_info,
		  const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    FLOAT_PARAMS *cutoff_percents = (FLOAT_PARAMS *) tm->meas_params;
    long *cutoffs;    /* cutoffs expressed in num rel docs instead of percents*/
    long current_cut; /* current index into cutoffs */
    RANK_REL rr;
    long rel_so_far;
    long i;
    double precis, int_precis;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rr))
	return (UNDEF);

    /* translate percentage of rels as given in the measure params, to
       an actual cutoff number of docs.  Note addition of 0.9 
       means the default 11 percentages should have same cutoffs as
       historical MAP implementations (eg, old trec_eval) */
    if (NULL == (cutoffs = Malloc (cutoff_percents->num_params, long)))
	return (UNDEF);
    for (i = 0; i < cutoff_percents->num_params; i++)
	cutoffs[i] = (long) (cutoff_percents->param_values[i] * rr.num_rel+0.9);

    current_cut = cutoff_percents->num_params - 1;
    while (current_cut > 0 && cutoffs[current_cut] > rr.num_rel_ret)
	current_cut--;

    /* Loop over all retrieved docs in reverse order.  Needs to be
       reverse order since are calcualting interpolated precision.
       Int_Prec (X) defined to be MAX (Prec (Y)) for all Y >= X. */
    precis = (double) rr.num_rel_ret / (double) rr.num_ret;
    int_precis = precis;
    rel_so_far = rr.num_rel_ret;
    for (i = rr.num_ret; i > 0 && rel_so_far > 0; i--) {
	precis = (double) rel_so_far / (double) i;
	if (int_precis < precis)
	    int_precis = precis;
	if (rr.results_rel_list[i-1] >= epi->relevance_level) {
            while (current_cut >= 0 && rel_so_far == cutoffs[current_cut]) {
		eval->values[tm->eval_index+current_cut].value = int_precis;
                current_cut--;
            }
            rel_so_far--;
	}
    }

    while (current_cut >= 0) {
	eval->values[tm->eval_index+current_cut].value = int_precis;
	current_cut--;
    }

    (void) Free (cutoffs);

    return (1);
}
