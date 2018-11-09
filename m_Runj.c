/* 
   by Ian Soboroff, NIST
   This code is adapted from m_Rprec.c, but itself is in the public
   domain.
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "trec_format.h"

static int 
te_calc_Runj (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	      const TREC_MEAS *tm, TREC_EVAL *eval);

/* See trec_eval.h for definition of TREC_MEAS */
TREC_MEAS te_meas_Runj =
    {"Runj",
     "    Fraction of the ranking that is unjudged (either \n\
     unpooled or pooled and unjudged) at rank R, which is the \n\
     number of known relevant documents.  Note that in contrast\n\
     to R-Precision, the number R here is just used as a cutoff\n\
     and not a normalization\n.",
     te_init_meas_s_float,
     te_calc_Runj,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     te_print_single_meas_s_float,
     te_print_final_meas_s_float,
     NULL, -1};

static int 
te_calc_Runj (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	      const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RES_RELS res_rels;
    long num_to_look_at;
    long unj_so_far;
    long i;

    if (UNDEF == te_form_res_rels (epi, rel_info, results, &res_rels))
	return (UNDEF);

    unj_so_far = 0;
    num_to_look_at = MIN (res_rels.num_ret, res_rels.num_rel);
    if (0 == num_to_look_at)
	return (0);

    for (i = 0; i < num_to_look_at; i++) {
	if (res_rels.results_rel_list[i] < 0)
	    unj_so_far++;
    }
    eval->values[tm->eval_index].value =
	(double) unj_so_far / (double) num_to_look_at;
    return (1);
}
