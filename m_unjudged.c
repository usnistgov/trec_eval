/*
   Ian Soboroff, 2023
   This file is a creation of an employee of the US federal government
   and is in the public domain.
*/
#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "trec_format.h"

static int
te_calc_unj(const EPI * epi,
            const REL_INFO * rel_info,
            const RESULTS * results,
            const TREC_MEAS * tm, TREC_EVAL * eval);

static long long_cutoff_array[] = { 5, 10, 20 };

static PARAMS default_unj_cutoffs = {
    NULL, sizeof(long_cutoff_array) / sizeof(long_cutoff_array[0]),
    &long_cutoff_array[0]
};

/* See trec_eval.h for definition of TREC_MEAS */
TREC_MEAS te_meas_unj = { "unj",
    "    Unjudged at cutoffs\n\
    The fraction of unjudged documents measured at various doc level\n\
    cutoffs in the ranking.  If the cutoff is larger than the number\n\
    of docs retrieved, then it is assumed irrelevant (not unjudged)\n\
    docs fill in the rest.  For example, if a method retrieves 15\n\
    docs of which 4 have a qrel value that is not '-1', then\n\
    unj20 is 0.2 (4/20).\n\
    Unjudged at cutoff is a \"red flag\" measure: high values at low\n\
    cutoffs indicates that the run is poorly covered by the test\n\
    collection, either because the run is unusually bad, or finds\n\
    lots of unjudged relevant documents.\n",
    te_init_meas_a_double_cut_long,
    te_calc_unj,
    te_acc_meas_a_cut,
    te_calc_avg_meas_a_cut,
    te_print_single_meas_a_cut,
    te_print_final_meas_a_cut,
    (void *) &default_unj_cutoffs, -1
};

static int
te_calc_unj(const EPI * epi,
            const REL_INFO * rel_info,
            const RESULTS * results,
            const TREC_MEAS * tm, TREC_EVAL * eval)
{
    long *cutoffs = (long *) tm->meas_params->param_values;
    long cutoff_index = 0;
    long i;
    RES_RELS res_rels;
    long unj_so_far = 0;

    if (UNDEF == te_form_res_rels(epi, rel_info, results, &res_rels))
        return (UNDEF);

    for (i = 0; i < res_rels.num_ret; i++) {
        if (i == cutoffs[cutoff_index]) {
            /* Calculate previous cutoff threshold.
               Note all guaranteed to be positive by init_meas */
            eval->values[tm->eval_index + cutoff_index].value =
                (double) unj_so_far / (double) i;
            if (++cutoff_index == tm->meas_params->num_params)
                break;
        }
        if (res_rels.results_rel_list[i] == RELVALUE_NONPOOL ||
            res_rels.results_rel_list[i] == RELVALUE_UNJUDGED)
            unj_so_far++;
    }
    /* calculate values for those cutoffs not achieved */
    while (cutoff_index < tm->meas_params->num_params) {
        eval->values[tm->eval_index + cutoff_index].value =
            (double) unj_so_far / (double) cutoffs[cutoff_index];
        cutoff_index++;
    }
    return (1);
}
