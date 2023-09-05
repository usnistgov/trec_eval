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
te_calc_Rndcg(const EPI * epi, const REL_INFO * rel_info,
              const RESULTS * results, const TREC_MEAS * tm, TREC_EVAL * eval);
static PARAMS default_ndcg_gains = { NULL, 0, NULL };

/* See trec_eval.h for definition of TREC_MEAS */
TREC_MEAS te_meas_Rndcg = { "Rndcg",
    "    Normalized Discounted Cumulative Gain at R levels\n\
    Experimental measure\n\
    Compute a traditional nDCG measure according to Jarvelin and\n\
    Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002), averaged at the various\n\
    R level points. The R levels are the number of docs at each non-negative\n\
    gain level in the judgments, with the gain levels sorted in decreasing\n\
    order. Thus if there are 5 docs with gain_level 3, 3 with gain 2, 10\n\
    with gain 1, and 50 with gain 0, then \n\
    Rndcg = 1/4 (ndcg_at_5 + ndcg_at_8 + ndcg_at_18 + ndcg_at_68).\n\
    In this formulation, all unjudged docs have gain 0.0, and thus there is\n\
    a final implied R-level change at num_retrieved.\n\
    Idea behind Rndcg, is that the expected value of ndcg is a smoothly\n\
    decreasing function, with discontinuities upward at each transistion\n\
    between positive gain levels in the ideal ndcg.  Once the gain level \n\
    becomes 0, the expected value of ndcg then increases until all docs are\n\
    retrieved. Thus averaging ndcg is problematic, because these transistions\n\
    occur at different points for each topic.  Since it is not unusual for\n\
    ndcg to start off near 1.0, decrease to 0.25, and then increase to 0.75\n\
    at various cutoffs, the points at which ndcg is measured are important.\n\
    \n\
    Gain values are set to the appropriate relevance level by default.  \n\
    The default gain can be overridden on the command line by having \n\
    comma separated parameters 'rel_level=gain'.\n\
    Eg, 'trec_eval -m Rndcg.1=3.5,2=9.0,4=7.0 ...'\n\
    will give gains 3.5, 9.0, 3.0, 7.0 for relevance levels 1,2,3,4\n\
    respectively (level 3 remains at the default).\n\
    Gains are allowed to be 0 or negative, and relevance level 0\n\
    can be given a gain.\n",
    te_init_meas_s_double_p_pair,
    te_calc_Rndcg,
    te_acc_meas_s,
    te_calc_avg_meas_s,
    te_print_single_meas_s_double,
    te_print_final_meas_s_double_p,
    &default_ndcg_gains, -1
};

static int
te_calc_Rndcg(const EPI * epi, const REL_INFO * rel_info,
              const RESULTS * results, const TREC_MEAS * tm, TREC_EVAL * eval)
{
    RES_RELS res_rels;
    double results_gain, results_dcg;
    double old_ideal_gain, ideal_gain, ideal_dcg;
    double sum = 0.0;
    long num_changed_ideal_gain;
    long cur_level, num_at_level;
    long i;
    GAINS gains;

    if (UNDEF == te_form_res_rels(epi, rel_info, results, &res_rels))
        return (UNDEF);

    if (UNDEF == setup_gains(tm, &res_rels, &gains))
        return (UNDEF);

    if (res_rels.num_rel == 0)
        return (0);

    results_dcg = 0.0;
    ideal_dcg = 0.0;
    cur_level = gains.num_gains - 1;
    ideal_gain = (cur_level >= 0) ? gains.rel_gains[cur_level].gain : 0.0;
    old_ideal_gain = ideal_gain;
    num_changed_ideal_gain = 0;
    num_at_level = 0;

    for (i = 0; i < res_rels.num_ret && ideal_gain > 0.0; i++) {
        /* Calculate change in results dcg */
        results_gain = get_gain(res_rels.results_rel_list[i], &gains);
        /* Calculate change in ideal dcg */
        num_at_level++;
        while (cur_level >= 0 &&
               num_at_level > gains.rel_gains[cur_level].num_at_level) {
            num_at_level = 1;
            cur_level--;
            ideal_gain =
                (cur_level >= 0) ? gains.rel_gains[cur_level].gain : 0.0;
        }
        /* See if at boundary for changed ideal gain - if so, calc ndcg at
           this point for later averaging */
        if (old_ideal_gain != ideal_gain) {
            if (ideal_dcg > 0.0) {
                sum += results_dcg / ideal_dcg;
                num_changed_ideal_gain++;
            }
            old_ideal_gain = ideal_gain;
        }
        if (results_gain != 0)
            /* Note: i+2 since doc i has rank i+1 */
            results_dcg += results_gain / log2((double) (i + 2));
        if (ideal_gain > 0.0)
            ideal_dcg += ideal_gain / log2((double) (i + 2));
        if (epi->debug_level > 0)
            printf("Rndcg: %ld %ld %3.1f %6.4f %3.1f %6.4f %6.4f\n",
                   i, cur_level, results_gain, results_dcg,
                   ideal_gain, ideal_dcg, sum);
    }
    if (i < res_rels.num_ret) {
        while (i < res_rels.num_ret) {
            /* Calculate change in results dcg */
            results_gain = get_gain(res_rels.results_rel_list[i], &gains);
            if (results_gain != 0)
                results_dcg += results_gain / log2((double) (i + 2));
            if (epi->debug_level > 0)
                printf("Rndcg: %ld %ld %3.1f %6.4f %3.1f %6.4f\n",
                       i, cur_level, results_gain, results_dcg, 0.0, ideal_dcg);
            i++;
        }
        if (ideal_dcg > 0.0) {
            sum += results_dcg / ideal_dcg;
            num_changed_ideal_gain++;
        }
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
        /* See if at boundary for changed ideal gain - if so, calc ndcg at
           this point for later averaging */
        if (old_ideal_gain != ideal_gain) {
            if (ideal_dcg > 0.0) {
                sum += results_dcg / ideal_dcg;
                num_changed_ideal_gain++;
            }
            old_ideal_gain = ideal_gain;
        }
        if (ideal_gain > 0.0)
            ideal_dcg += ideal_gain / log2((double) (i + 2));
        if (epi->debug_level > 0)
            printf("Rndcg: %ld %ld %3.1f %6.4f %3.1f %6.4f\n",
                   i, cur_level, 0.0, results_dcg, ideal_gain, ideal_dcg);
        i++;
    }

    eval->values[tm->eval_index].value = sum / num_changed_ideal_gain;

    Free(gains.rel_gains);
    return (1);
}
