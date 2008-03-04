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

/*      "    Mean Average Precision\n\
    Precision measured after each relevant doc is retrieved, then averaged\n\
    for the topic, and then averaged over topics (if more than one).\n\
    This is the main single-valued number used to compare the entire rankings\n\
    of two or more retrieval methods.  It has proven in practice to be useful\n\
    and robust.\n\
    The name of the measure is unfortunately inaccurate since it is \n\
    calculated for a single topic (and thus don't want both 'mean' and\n\
    'average') but was dictated by common usage and the need to distiguish\n\
    map from Precision averaged over topics (I had to give up my attempts to\n\
    call it something else!)\n\
    History: Developed by Chris Buckley after TREC 1.\n\
    Cite: 'Retrieval System Evaluation', Chris Buckley and Ellen Voorhees.\n\
    Chapter 3 in TREC: Experiment and Evaluation in Information Retrieval\n\
    edited by Ellen Voorhees and Donna Harman.  MIT Press 2005\n",
*/

int 
te_calc_map (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	     const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;
    double sum;
    long rel_so_far;
    long i;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    rel_so_far = 0;
    sum = 0.0;
    for (i = 0; i < rank_rel.num_ret; i++) {
	if (rank_rel.results_rel_list[i] >= epi->relevance_level) {
	    rel_so_far++;
	    sum += (double) rel_so_far / (double) (i + 1);
	}
    }
    /* Average over the rel docs */
    if (rel_so_far) {
	eval->values[tm->eval_index].value = 
	    sum / (double) rank_rel.num_rel;
    }
    return (1);
}
