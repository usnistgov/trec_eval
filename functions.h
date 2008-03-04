/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/
#ifndef FUNCTIONSH
#define FUNCTIONSH

#include "trec_eval.h"
/* Utility functions */
void * te_chk_and_malloc (void *ptr, long *current_bound,
			  const long needed, const size_t size);
void * te_chk_and_realloc (void *ptr, long *current_bound,
			   const long needed, const int size);


/* ------------------- Generic Routines for Measures ------------------------ */

/* -------- Initialize measure -------- */
/* Measure does not require initialization or storage */
int te_init_meas_empty (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
/* Measure is a single float measure, no parameters, always printed */
int te_init_meas_s_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
/* Measure is a single float measure, no parameters, printed only in summary */
int te_init_meas_s_float_summ (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
/* Measure is a single long measure, no parameters. always printed */
int te_init_meas_s_long (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
/* Measure is a single long measure, no parameters. printed only in summary */
int te_init_meas_s_long_summ (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
/* Measure is a float array with long cutoffs */
int te_init_meas_a_float_cut_long (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
/* Measure is a float array with float cutoffs */
int te_init_meas_a_float_cut_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
/* Measure is a single float with float params */
int te_init_meas_s_float_p_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
/* Measure is a single float with paired name=float params */
int te_init_meas_s_float_p_pair (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);


/* -------- Accumulate Measure -------- */
/* Measure does not require accumulation */
int te_acc_meas_empty (const EPI *epi, const TREC_MEAS *tm,
		       const TREC_EVAL *q_eval, TREC_EVAL *accum_eval);
/* Accumulate a single value */
int te_acc_meas_s (const EPI *epi, const TREC_MEAS *tm,
		   const TREC_EVAL *q_eval, TREC_EVAL *accum_eval);
/* Accumulate an array of values (one per cutoff) */
int te_acc_meas_a_cut (const EPI *epi, const TREC_MEAS *tm,
		       const TREC_EVAL *q_eval, TREC_EVAL *accum_eval);

/* ----- Calculate Average measure ---- */
/* Measure does not require final averaging */
int te_calc_avg_meas_empty (const EPI *epi, const TREC_MEAS *tm,
			    TREC_EVAL *eval);
/* Measure is a single value with averaging */
int te_calc_avg_meas_s (const EPI *epi, const TREC_MEAS *tm,
			TREC_EVAL *accum_eval);
/* Measure is an array of values (one per cutoff) with averaging */
int te_calc_avg_meas_a_cut (const EPI *epi, const TREC_MEAS *tm,
			    TREC_EVAL *accum_eval);
/* Measure is a single value using geometric mean */
int te_calc_avg_meas_s_gm (const EPI *epi, const TREC_MEAS *tm,
			   TREC_EVAL *accum_eval);

/* ------------------- Single Measure Routines ------------------------ */
int te_calc_map (const EPI *epi, const REL_INFO *rel_info,
		 const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval);
int te_calc_P (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	       const TREC_MEAS *tm, TREC_EVAL *eval);
int te_calc_num_q (const EPI *epi, const REL_INFO *rel_info,
		   const RESULTS *results, const TREC_MEAS *tm,
		   TREC_EVAL *eval);
int te_calc_avg_num_q (const EPI *epi, const TREC_MEAS *tm,
		       TREC_EVAL *accum_eval);
int te_calc_num_ret (const EPI *epi, const REL_INFO *rel_info,
		     const RESULTS *results, const TREC_MEAS *tm,
		     TREC_EVAL *eval);
int te_calc_num_rel (const EPI *epi, const REL_INFO *rel_info,
		     const RESULTS *results, const TREC_MEAS *tm,
		     TREC_EVAL *eval);
int te_calc_num_rel_ret (const EPI *epi, const REL_INFO *rel_info, 
			 const RESULTS *results, const TREC_MEAS *tm,
			 TREC_EVAL *eval);
int te_calc_gm_map (const EPI *epi, const REL_INFO *rel_info,
		    const RESULTS *results, const TREC_MEAS *tm,
		    TREC_EVAL *eval);
int te_calc_Rprec (const EPI *epi, const REL_INFO *rel_info,
		   const RESULTS *results, const TREC_MEAS *tm,
		   TREC_EVAL *eval);
int te_calc_recip_rank (const EPI *epi, const REL_INFO *rel_info,
			const RESULTS *results, const TREC_MEAS *tm,
			TREC_EVAL *eval);
int te_calc_bpref (const EPI *epi, const REL_INFO *rel_info,
		   const RESULTS *results, const TREC_MEAS *tm,
		   TREC_EVAL *eval);
int te_calc_gm_bpref (const EPI *epi, const REL_INFO *rel_info,
		      const RESULTS *results, const TREC_MEAS *tm,
		      TREC_EVAL *eval);
int te_calc_infap (const EPI *epi, const REL_INFO *rel_info,
		   const RESULTS *results, const TREC_MEAS *tm,
		   TREC_EVAL *eval);
int te_calc_iprec_at_recall (const EPI *epi, const REL_INFO *rel_info,
			     const RESULTS *results, const TREC_MEAS *tm,
			     TREC_EVAL *eval);
int te_calc_recall (const EPI *epi, const REL_INFO *rel_info,
		    const RESULTS *results, const TREC_MEAS *tm,
		    TREC_EVAL *eval);
int te_calc_Rprec_mult (const EPI *epi, const REL_INFO *rel_info,
			const RESULTS *results, const TREC_MEAS *tm,
			TREC_EVAL *eval);
int te_calc_utility (const EPI *epi, const REL_INFO *rel_info,
		     const RESULTS *results, const TREC_MEAS *tm,
		     TREC_EVAL *eval);
int te_calc_11ptavg (const EPI *epi, const REL_INFO *rel_info,
		     const RESULTS *results, const TREC_MEAS *tm,
		     TREC_EVAL *eval);
int te_calc_ndcg (const EPI *epi, const REL_INFO *rel_info,
		  const RESULTS *results, const TREC_MEAS *tm,
		  TREC_EVAL *eval);
int te_calc_ndcg_p (const EPI *epi, const REL_INFO *rel_info,
		    const RESULTS *results, const TREC_MEAS *tm,
		    TREC_EVAL *eval);
int te_calc_ndcg_cut (const EPI *epi, const REL_INFO *rel_info,
		      const RESULTS *results, const TREC_MEAS *tm,
		      TREC_EVAL *eval);
int te_calc_rel_P (const EPI *epi, const REL_INFO *rel_info,
		   const RESULTS *results, const TREC_MEAS *tm,
		   TREC_EVAL *eval);
int te_calc_success (const EPI *epi, const REL_INFO *rel_info,
		     const RESULTS *results, const TREC_MEAS *tm,
		     TREC_EVAL *eval);
int te_calc_set_P (const EPI *epi, const REL_INFO *rel_info,
		   const RESULTS *results, const TREC_MEAS *tm,
		   TREC_EVAL *eval);
int te_calc_set_relative_P (const EPI *epi, const REL_INFO *rel_info, 
			    const RESULTS *results, const TREC_MEAS *tm,
			    TREC_EVAL *eval);
int te_calc_set_recall (const EPI *epi, const REL_INFO *rel_info,
			const RESULTS *results, const TREC_MEAS *tm,
			TREC_EVAL *eval);
int te_calc_set_map (const EPI *epi, const REL_INFO *rel_info,
		     const RESULTS *results, const TREC_MEAS *tm,
		     TREC_EVAL *eval);
int te_calc_set_F (const EPI *epi, const REL_INFO *rel_info,
		   const RESULTS *results, const TREC_MEAS *tm,
		   TREC_EVAL *eval);
int te_calc_num_nonrel_judged_ret (const EPI *epi, const REL_INFO *rel_info,
				   const RESULTS *results,
				   const TREC_MEAS *tm, TREC_EVAL *eval);
int te_calc_prefs_simp (const EPI *epi, const REL_INFO *rel_info,
			const RESULTS *results, const TREC_MEAS *tm,
			TREC_EVAL *eval);
int te_calc_prefs_pair (const EPI *epi, const REL_INFO *rel_info,
			const RESULTS *results, const TREC_MEAS *tm,
			TREC_EVAL *eval);
int te_calc_prefs_avgjg (const EPI *epi, const REL_INFO *rel_info,
			 const RESULTS *results, const TREC_MEAS *tm,
			 TREC_EVAL *eval);
int te_calc_prefs_avgjg_Rnonrel (const EPI *epi, const REL_INFO *rel_info,
				 const RESULTS *results, const TREC_MEAS *tm,
				 TREC_EVAL *eval);
int te_calc_prefs_simp_ret (const EPI *epi, const REL_INFO *rel_info,
			    const RESULTS *results, const TREC_MEAS *tm,
			    TREC_EVAL *eval);
int te_calc_prefs_pair_ret (const EPI *epi, const REL_INFO *rel_info,
			    const RESULTS *results, const TREC_MEAS *tm,
			    TREC_EVAL *eval);
int te_calc_prefs_avgjg_ret (const EPI *epi, const REL_INFO *rel_info,
			     const RESULTS *results, const TREC_MEAS *tm,
			     TREC_EVAL *eval);
int te_calc_prefs_avgjg_Rnonrel_ret (const EPI *epi, const REL_INFO *rel_info,
				     const RESULTS *results,
				     const TREC_MEAS *tm,
				     TREC_EVAL *eval);
int te_calc_prefs_simp_imp (const EPI *epi, const REL_INFO *rel_info,
			    const RESULTS *results, const TREC_MEAS *tm,
			    TREC_EVAL *eval);
int te_calc_prefs_pair_imp (const EPI *epi, const REL_INFO *rel_info,
			    const RESULTS *results, const TREC_MEAS *tm,
			    TREC_EVAL *eval);
int te_calc_prefs_avgjg_imp (const EPI *epi, const REL_INFO *rel_info,
			     const RESULTS *results, const TREC_MEAS *tm,
			     TREC_EVAL *eval);
int te_calc_prefs_num_prefs_poss (const EPI *epi, const REL_INFO *rel_info,
				  const RESULTS *results, const TREC_MEAS *tm,
				  TREC_EVAL *eval);
int te_calc_prefs_num_prefs_ful (const EPI *epi, const REL_INFO *rel_info,
				 const RESULTS *results, const TREC_MEAS *tm,
				 TREC_EVAL *eval);
int te_calc_prefs_num_prefs_ful_ret (const EPI *epi, const REL_INFO *rel_info,
				     const RESULTS *results,
				     const TREC_MEAS *tm,
				     TREC_EVAL *eval);
int te_calc_map_avgjg (const EPI *epi, const REL_INFO *rel_info,
		       const RESULTS *results, const TREC_MEAS *tm,
		       TREC_EVAL *eval);
int te_calc_P_avgjg (const EPI *epi, const REL_INFO *rel_info,
		     const RESULTS *results, const TREC_MEAS *tm,
		     TREC_EVAL *eval);
int te_calc_Rprec_mult_avgjg (const EPI *epi, const REL_INFO *rel_info,
			      const RESULTS *results, const TREC_MEAS *tm,
			      TREC_EVAL *eval);

#endif /* FUNCTIONSH */
