/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/
#ifndef FUNCTIONSH
#define FUNCTIONSH

#include "trec_eval.h"
#include "trec_format.h"

/* Utility functions */
void *te_chk_and_malloc(void *ptr, long *current_bound,
                        const long needed, const size_t size);
void *te_chk_and_realloc(void *ptr, long *current_bound,
                         const long needed, const int size);
/* Functions for dealing with zscores */
int te_get_zscores(const EPI * epi, const char *zscores_file,
                   ALL_ZSCORES * zscores);
int te_get_zscores_cleanup();
int te_convert_to_zscore(const ALL_ZSCORES * all_zscores, TREC_EVAL * q_eval);
/* Functions for dealing with gain values, defined in gain_init.c */
int setup_gains(const TREC_MEAS * tm, const RES_RELS * res_rels, GAINS * gains);
double get_gain(const long rel_level, const GAINS * gains);
int comp_rel_gain();

/* ------------------- Generic Routines for Measures ------------------------ */

/* -------- Initialize measure -------- */
/* Code is in meas_init.c */
/* Measure does not require initialization or storage */
int te_init_meas_empty(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval);
/* Measure is a single double measure, no parameters */
int te_init_meas_s_double(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval);
/* Measure is a single long measure, no parameters. */
int te_init_meas_s_long(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval);
/* Measure is a double array with long cutoffs */
int te_init_meas_a_double_cut_long(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval);
/* Measure is a double array with double cutoffs */
int te_init_meas_a_double_cut_double(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval);
/* Measure is a single double with double params */
int te_init_meas_s_double_p_double(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval);
/* Measure is a single double with paired name=double params */
int te_init_meas_s_double_p_pair(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval);


/* -------- Accumulate Measure -------- */
/* Code is in meas_acc.c */
/* Measure does not require accumulation */
int te_acc_meas_empty(const EPI * epi, const TREC_MEAS * tm,
                      const TREC_EVAL * q_eval, TREC_EVAL * accum_eval);
/* Accumulate a single value */
int te_acc_meas_s(const EPI * epi, const TREC_MEAS * tm,
                  const TREC_EVAL * q_eval, TREC_EVAL * accum_eval);
/* Accumulate an array of values (one per cutoff) */
int te_acc_meas_a_cut(const EPI * epi, const TREC_MEAS * tm,
                      const TREC_EVAL * q_eval, TREC_EVAL * accum_eval);

/* ----- Calculate Average measure ---- */
/* Code is in meas_calc_avg.c */
/* Measure does not require final averaging */
int te_calc_avg_meas_empty(const EPI * epi, const TREC_MEAS * tm,
                           const ALL_REL_INFO * all_rel_info, TREC_EVAL * eval);
/* Measure is a single value with averaging */
int te_calc_avg_meas_s(const EPI * epi, const TREC_MEAS * tm,
                       const ALL_REL_INFO * all_rel_info,
                       TREC_EVAL * accum_eval);
/* Measure is an array of values (one per cutoff) with averaging */
int te_calc_avg_meas_a_cut(const EPI * epi, const TREC_MEAS * tm,
                           const ALL_REL_INFO * all_rel_info,
                           TREC_EVAL * accum_eval);
/* Measure is a single value using geometric mean */
int te_calc_avg_meas_s_gm(const EPI * epi, const TREC_MEAS * tm,
                          const ALL_REL_INFO * all_rel_info,
                          TREC_EVAL * accum_eval);

/* ----- Print single query for measure ---- */
/* Code is in meas_print_single.c */
/* Measure does not require printing */
int te_print_single_meas_empty(const EPI * epi, const TREC_MEAS * tm,
                               const TREC_EVAL * eval);
/* Measure is a single double measure with no parameters, */
int te_print_single_meas_s_double(const EPI * epi, const TREC_MEAS * tm,
                                 const TREC_EVAL * eval);
/* Measure is a single long measure with no parameters. */
int te_print_single_meas_s_long(const EPI * epi, const TREC_MEAS * tm,
                                const TREC_EVAL * eval);
/* Measure is a double array with cutoffs */
int te_print_single_meas_a_cut(const EPI * epi, const TREC_MEAS * tm,
                               const TREC_EVAL * eval);


/* ----- Print final query for measure ---- */
/* Code is in meas_print_final.c */
/* Measure does not require printing or storage */
int te_print_final_meas_empty(const EPI * epi, TREC_MEAS * tm,
                              TREC_EVAL * eval);
/* Measure is a single double measure with no parameters */
int te_print_final_meas_s_double(const EPI * epi, TREC_MEAS * tm,
                                TREC_EVAL * eval);
/* Measure is a single long measure with no parameters. */
int te_print_final_meas_s_long(const EPI * epi, TREC_MEAS * tm,
                               TREC_EVAL * eval);
/* Measure is a double array with cutoffs */
int te_print_final_meas_a_cut(const EPI * epi, TREC_MEAS * tm,
                              TREC_EVAL * eval);
/* Measure is a single double with double params */
int te_print_final_meas_s_double_p(const EPI * epi, TREC_MEAS * tm,
                                  TREC_EVAL * eval);

#endif                          /* FUNCTIONSH */
