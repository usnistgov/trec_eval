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
te_calc_rbp_resid(const EPI * epi, const REL_INFO * rel_info,
                  const RESULTS * results, const TREC_MEAS * tm,
                  TREC_EVAL * eval);
static PARAMS default_rbp_params = { NULL, 0, NULL };

/* See trec_eval.h for definition of TREC_MEAS */
TREC_MEAS te_meas_rbp_resid = { "rbp_resid",
    "    Rank-Biased Precision residual\n\
    RBP measures the rate at which utility is gained by a user \n\
    working at a given degree of persistence.  By default the \n\
    persistence parameter p = 0.9, causing RBP to model something \n\
    close to P_10.  Setting p = 0.95 or 0.98 gets closer to average \n\
    precision.  Lower p values cause the metric to resemble precision \n\
    at high cutoff.\n\
    rbp_resid, the residual of RBP, captures the uncertainty in the RBP\n\
    metric that arises from unjudged documents in the ranked list.\n\
    rbp_resid computes the maximum upward score shift in the event that\n\
    all retrieved unjudged documents had maximum gain.\n\
    Cite: Moffat, Alistair and Justin Zobel, \"Rank-Biased Precision for \n\
    Measurement of Retrieval Effectiveness.\"  ACM Transactions on\n\
    Information Systems, vol. 27, no. 1, article 2, publication date\n\
    December, 2008.  http://doi.acm.org/10.1145/1416950.1416952\n",
    te_init_meas_s_double_p_pair,
    te_calc_rbp_resid,
    te_acc_meas_s,
    te_calc_avg_meas_s,
    te_print_single_meas_s_double,
    te_print_final_meas_s_double_p,
    &default_rbp_params, -1
};

static int
te_calc_rbp_resid(const EPI * epi, const REL_INFO * rel_info,
                  const RESULTS * results, const TREC_MEAS * tm,
                  TREC_EVAL * eval)
{
    RES_RELS res_rels;
    double sum;
    double p = 0.9;             /* default value, settable with param */
    double cur_p;
    long unj_so_far;
    long i;
    DOUBLE_PARAM_PAIR *pairs;
    int num_pairs;

    if (UNDEF == te_form_res_rels(epi, rel_info, results, &res_rels))
        return (UNDEF);

    if (tm->meas_params) {
        pairs = (DOUBLE_PARAM_PAIR *) tm->meas_params->param_values;
        num_pairs = tm->meas_params->num_params;
        for (i = 0; i < num_pairs; i++) {
            if (0 == strcmp(pairs[i].name, "p"))
                p = (double) pairs[i].value;
        }
    }

    unj_so_far = 0;
    sum = 0.0;
    cur_p = 1.0;                /* p^0 */
    for (i = 0; i < res_rels.num_ret; i++) {
        if (res_rels.results_rel_list[i] < 0) {
            unj_so_far++;
            sum += cur_p;
        }
        cur_p = cur_p * p;
    }

    if (unj_so_far) {
        eval->values[tm->eval_index].value = cur_p + (1 - p) * sum;
    }
    return (1);
}
