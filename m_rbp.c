/* 
   Written by Ian Soboroff, NIST, 2016.
   As a government work this file is in the public domain.
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "trec_format.h"

static int
te_calc_rbp (const EPI *epi, const REL_INFO *rel_info,
	     const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval);

/* See trec_eval.h for definition of TREC_MEAS */
TREC_MEAS te_meas_rbp =  {"rbp",
     "    Rank-Biased Precision\n\
    This description needs updating (it's the text for AP)\n\
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
    Cite: Moffat, Alistair and Justin Zobel, \"Rank-Biased Precision for \n\
    Measurement of Retrieval Effectiveness.\"  ACM Transactions on\n\
    Information Systems, vol. 27, no. 1, article 2, publication date\n\
    December, 2008.  http://doi.acm.org/10.1145/1416950.1416952\n",
     te_init_meas_s_float,
     te_calc_rbp,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     te_print_single_meas_s_float,
     te_print_final_meas_s_float,
     NULL, -1};

static int 
te_calc_rbp (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	     const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RES_RELS res_rels;
    double sum;
    double p = 0.9; /* setting constant p for development */
    double cur_p;
    long rel_so_far;
    long i;

    if (UNDEF == te_form_res_rels (epi, rel_info, results, &res_rels))
	return (UNDEF);

    rel_so_far = 0;
    sum = 0.0;
    cur_p = 1.0;  /* p^0 */
    for (i = 0; i < res_rels.num_ret; i++) {
	if (res_rels.results_rel_list[i] >= epi->relevance_level) {
	    rel_so_far++;
	    sum += (double) res_rels.results_rel_list[i] * cur_p;
	}
        cur_p = cur_p * p;
    }

    if (rel_so_far) {
	eval->values[tm->eval_index].value = (1.0 - p) * sum;
    }
    return (1);
}
