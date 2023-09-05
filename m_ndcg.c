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
double log2(double x);

static int
te_calc_ndcg(const EPI * epi, const REL_INFO * rel_info,
             const RESULTS * results, const TREC_MEAS * tm, TREC_EVAL * eval);
static PARAMS default_ndcg_gains = { NULL, 0, NULL };

TREC_MEAS te_meas_dcg = { "dcg",
    "    Discounted Cumulative Gain",
    te_init_meas_s_double_p_pair,
    te_calc_ndcg,
    te_acc_meas_s,
    te_calc_avg_meas_s,
    te_print_single_meas_s_double,
    te_print_final_meas_s_double_p,
    &default_ndcg_gains, -1
};

TREC_MEAS te_meas_ideal_dcg = { "ideal_dcg",
    "    Ideal Discounted Cumulative Gain",
    te_init_meas_s_double_p_pair,
    te_calc_ndcg,
    te_acc_meas_s,
    te_calc_avg_meas_s,
    te_print_single_meas_s_double,
    te_print_final_meas_s_double_p,
    &default_ndcg_gains, -1
};

/* See trec_eval.h for definition of TREC_MEAS */
TREC_MEAS te_meas_ndcg = { "ndcg",
    "    Normalized Discounted Cumulative Gain\n\
    Compute a traditional nDCG measure according to Jarvelin and\n\
    Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002)\n\
    Gain values are set to the appropriate relevance level by default.  \n\
    The default gain can be overridden on the command line by having \n\
    comma separated parameters 'rel_level=gain'.\n\
    Eg, 'trec_eval -m ndcg.1=3.5,2=9.0,4=7.0 ...'\n\
    will give gains 3.5, 9.0, 3.0, 7.0 for relevance levels 1,2,3,4\n\
    respectively (level 3 remains at the default).\n\
    Gains are allowed to be 0 or negative, and relevance level 0\n\
    can be given a gain.\n\
    Based on an implementation by Ian Soboroff\n",
    te_init_meas_s_double_p_pair,
    te_calc_ndcg,
    te_acc_meas_s,
    te_calc_avg_meas_s,
    te_print_single_meas_s_double,
    te_print_final_meas_s_double_p,
    &default_ndcg_gains, -1
};

static int
te_calc_ndcg(const EPI * epi, const REL_INFO * rel_info,
             const RESULTS * results, const TREC_MEAS * tm, TREC_EVAL * eval)
{
    RES_RELS res_rels;
    double results_gain, results_dcg;
    double ideal_gain, ideal_dcg;
    long cur_level, num_at_level;
    long i;
    GAINS gains;

    if (UNDEF == te_form_res_rels(epi, rel_info, results, &res_rels))
        return (UNDEF);

    if (UNDEF == setup_gains(tm, &res_rels, &gains))
        return (UNDEF);

    results_dcg = 0.0;
    ideal_dcg = 0.0;
    cur_level = gains.num_gains - 1;
    ideal_gain = (cur_level >= 0) ? gains.rel_gains[cur_level].gain : 0.0;
    num_at_level = 0;

    for (i = 0; i < res_rels.num_ret && ideal_gain > 0.0; i++) {
        /* Calculate change in results dcg */
        results_gain = get_gain(res_rels.results_rel_list[i], &gains);
        if (results_gain != 0)
            /* Note: i+2 since doc i has rank i+1 */
            results_dcg += results_gain / log2((double) (i + 2));
        /* Calculate change in ideal dcg */
        num_at_level++;
        while (cur_level >= 0 &&
               num_at_level > gains.rel_gains[cur_level].num_at_level) {
            num_at_level = 1;
            cur_level--;
            ideal_gain =
                (cur_level >= 0) ? gains.rel_gains[cur_level].gain : 0.0;
        }
        if (ideal_gain > 0.0)
            ideal_dcg += ideal_gain / log2((double) (i + 2));
        if (epi->debug_level > 0)
            printf("ndcg: %ld %ld %3.1f %6.4f %3.1f %6.4f\n",
                   i, cur_level, results_gain, results_dcg,
                   ideal_gain, ideal_dcg);
    }
    while (i < res_rels.num_ret) {
        /* Calculate change in results dcg */
        results_gain = get_gain(res_rels.results_rel_list[i], &gains);
        if (results_gain != 0)
            results_dcg += results_gain / log2((double) (i + 2));
        if (epi->debug_level > 0)
            printf("ndcg: %ld %ld %3.1f %6.4f %3.1f %6.4f\n",
                   i, cur_level, results_gain, results_dcg, 0.0, ideal_dcg);
        i++;
    }
    while (ideal_gain > 0.0) {
        /* Calculate change in ideal dcg */
        num_at_level++;
        while (cur_level >= 0 &&
               num_at_level > gains.rel_gains[cur_level].num_at_level) {
            num_at_level = 1;
            cur_level--;
            ideal_gain =
                (cur_level >= 0) ? gains.rel_gains[cur_level].gain : 0.0;
        }
        if (ideal_gain > 0.0)
            ideal_dcg += ideal_gain / log2((double) (i + 2));
        if (epi->debug_level > 0)
            printf("ndcg: %ld %ld %3.1f %6.4f %3.1f %6.4f\n",
                   i, cur_level, 0.0, results_dcg, ideal_gain, ideal_dcg);
        i++;
    }

    if (strcmp(tm->name, "ndcg") == 0) {
        /* Compare sum to ideal NDCG */
        if (ideal_dcg > 0.0) {
            eval->values[tm->eval_index].value = results_dcg / ideal_dcg;
        }
    } else if (strcmp(tm->name, "dcg") == 0) {
        eval->values[tm->eval_index].value = results_dcg;
    } else if (strcmp(tm->name, "ideal_dcg") == 0) {
        eval->values[tm->eval_index].value = ideal_dcg;
    } else {
        fprintf(stderr, "trec_eval: Cannot initialize measure '%s'\n",
                tm->name);
        exit(2);
    }

    Free(gains.rel_gains);
    return (1);
}
