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

/*      "    Precision after R documents have been retrieved.\n\
    R is the total number of relevant docs for the topic.  \n\
    This is a good single point measure for an entire retrieval\n\
    ranking that averages well since each topic is being averaged\n\
    at an equivalent point in its result ranking.\n\
    Note that this is the point that Precision = Recall.\n\
    History: Originally developed for IR rankings by Chris Buckley\n\
    after TREC 1, but analogs were used in other disciplines previously.\n\
    (the point where P = R is an important one!)\n\
    Cite: 'Retrieval System Evaluation', Chris Buckley and Ellen Voorhees.\n\
    Chapter 3 in TREC: Experiment and Evaluation in Information Retrieval\n\
    edited by Ellen Voorhees and Donna Harman.  MIT Press 2005\n",
*/

int 
te_calc_Rprec (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	       const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;
    long num_to_look_at;
    long rel_so_far;
    long i;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    rel_so_far = 0;
    num_to_look_at = MIN (rank_rel.num_ret, rank_rel.num_rel);
    if (0 == num_to_look_at)
	return (0);

    for (i = 0; i < num_to_look_at; i++) {
	if (rank_rel.results_rel_list[i] >= epi->relevance_level)
	    rel_so_far++;
    }
    eval->values[tm->eval_index].value =
	(double) rel_so_far / (double) rank_rel.num_rel;
    return (1);
}
