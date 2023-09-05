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
te_calc_ndcg_p(const EPI * epi, const REL_INFO * rel_info,
               const RESULTS * results, const TREC_MEAS * tm, TREC_EVAL * eval);
static PARAMS default_ndcg_gains = { NULL, 0, NULL };

/* See trec_eval.h for definition of TREC_MEAS */
TREC_MEAS te_meas_ndcg_p = { "ndcg_p",
    "    Normalized Discounted Cumulative Gain\n\
    Compute a traditional nDCG measure according to Jarvelin and\n\
    Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002).\n\
    Gain values are set to the appropriate relevance level by default.  \n\
    The default gain can be overridden on the command line by having \n\
    comma separated parameters 'rel_level=gain'.\n\
    Eg, 'trec_eval -m ndcg_p.1=3.5,2=9.0,4=7.0 ...'\n\
    will give gains 3.5, 9.0, 3.0, 7.0 for relevance levels 1,2,3,4\n\
    respectively (level 3 remains at the default).\n\
    Gains are allowed to be 0 or negative, and relevance level 0\n\
    can be given a gain.\n\
    Based on an implementation by Ian Soboroff\n",
    te_init_meas_s_double_p_pair,
    te_calc_ndcg_p,
    te_acc_meas_s,
    te_calc_avg_meas_s,
    te_print_single_meas_s_double,
    te_print_final_meas_s_double_p,
    &default_ndcg_gains, -1
};

static int
te_calc_ndcg_p(const EPI * epi, const REL_INFO * rel_info,
               const RESULTS * results, const TREC_MEAS * tm, TREC_EVAL * eval)
{
    RES_RELS res_rels;
    double gain, sum;
    double ideal_dcg;           /* ideal discounted cumulative gain */
    long cur_lvl, lvl_count;
    long i;
    GAINS gains;

    if (UNDEF == te_form_res_rels(epi, rel_info, results, &res_rels))
        return (UNDEF);

    if (UNDEF == setup_gains(tm, &res_rels, &gains))
        return (UNDEF);

    sum = 0.0;
    for (i = 0; i < res_rels.num_ret; i++) {
        gain = get_gain(res_rels.results_rel_list[i], &gains);
        if (gain != 0) {
            if (i > 0)
                sum += gain / log2((double) (i + 1));
            else
                sum += gain;
            if (epi->debug_level > 0)
                printf("ndcg_p:%ld %3.1f %6.4f\n", i, gain, sum);
        }
    }
    /* Calculate ideal discounted cumulative gain for this topic */
    cur_lvl = gains.num_gains - 1;
    lvl_count = 0;
    ideal_dcg = 0.0;
    for (i = 0; i < gains.total_num_at_levels; i++) {
        lvl_count++;
        while (lvl_count > gains.rel_gains[cur_lvl].num_at_level) {
            lvl_count = 1;
            cur_lvl--;
            if (cur_lvl < 0 || gains.rel_gains[cur_lvl].gain <= 0.0)
                break;
        }
        if (cur_lvl < 0 || gains.rel_gains[cur_lvl].gain <= 0.0)
            break;
        gain = gains.rel_gains[cur_lvl].gain;
        if (i == 0)
            ideal_dcg += gain;
        else
            ideal_dcg += gain / (double) log2((double) (i + 1));
        if (epi->debug_level > 0)
            printf("ndcg_p:%ld %ld %3.1f %6.4f\n", i, cur_lvl, gain, ideal_dcg);
    }

    /* Compare sum to ideal NDCG */
    if (res_rels.num_rel_ret > 0) {
        eval->values[tm->eval_index].value = sum / ideal_dcg;
    }

    Free(gains.rel_gains);
    return (1);
}
