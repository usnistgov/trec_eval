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

/*     "    Geometric Mean Average Precision\n\
    This is the same measure as 'map' (see description of 'map') on an\n\
    individual topic, but the geometric mean is calculated when averaging\n\
    over topics.  This rewards methods that are more consistent over topics\n\
    as opposed to methods which do very well for some topics but very poorly\n\
    for others.\n\
    gm_ap is reported only in the summary over all topics, not for individual\n\
    topics.\n",
*/

int 
te_calc_gm_ap (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
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

    if (rel_so_far) {
	sum = sum / (double) rank_rel.num_rel;
    }
    eval->values[tm->eval_index].value =
	(double) log ((double)(MAX (sum, MIN_GEO_MEAN)));
    return (1);
}
