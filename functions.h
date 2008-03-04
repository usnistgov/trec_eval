#ifndef FUNCTIONSH
#define FUNCTIONSH

#include "trec_eval.h"
/* Utility functions */
void * te_chk_and_malloc (void *ptr, long *current_bound,
			  const long needed, const size_t size);
void * te_chk_and_realloc (void *ptr, long *current_bound,
			   const long needed, const int size);


/* ------------------- Generic Routines for Measures ------------------------ */

/* Initialize measure */
int te_init_meas_empty (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
int te_init_meas_s_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
int te_init_meas_s_float_summ (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
int te_init_meas_s_long (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
int te_init_meas_s_long_summ (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
int te_init_meas_a_float_cut_long (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
int te_init_meas_a_float_cut_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
int te_init_meas_s_float_p_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);
int te_init_meas_s_float_p_pair (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval);


/* Accumulate Measure */
int te_acc_meas_empty (const EPI *epi, const TREC_MEAS *tm,
		       const TREC_EVAL *q_eval, TREC_EVAL *accum_eval);
int te_acc_meas_s (const EPI *epi, const TREC_MEAS *tm,
		   const TREC_EVAL *q_eval, TREC_EVAL *accum_eval);
int te_acc_meas_a_cut_long (const EPI *epi, const TREC_MEAS *tm,
			    const TREC_EVAL *q_eval, TREC_EVAL *accum_eval);
int te_acc_meas_a_cut_float (const EPI *epi, const TREC_MEAS *tm,
			     const TREC_EVAL *q_eval, TREC_EVAL *accum_eval);

/* Calculate Average measure */
int te_calc_avg_meas_empty (const EPI *epi, const TREC_MEAS *tm,
			    TREC_EVAL *eval);
int te_calc_avg_meas_s (const EPI *epi, const TREC_MEAS *tm,
			TREC_EVAL *accum_eval);
int te_calc_avg_meas_a_cut_long (const EPI *epi, const TREC_MEAS *tm,
				 TREC_EVAL *accum_eval);
int te_calc_avg_meas_a_cut_float (const EPI *epi, const TREC_MEAS *tm,
				  TREC_EVAL *accum_eval);
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
int te_calc_gm_ap (const EPI *epi, const REL_INFO *rel_info,
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
int te_calc_ircl_prn (const EPI *epi, const REL_INFO *rel_info,
		      const RESULTS *results, const TREC_MEAS *tm,
		      TREC_EVAL *eval);
int te_calc_recall (const EPI *epi, const REL_INFO *rel_info,
		    const RESULTS *results, const TREC_MEAS *tm,
		    TREC_EVAL *eval);
int te_calc_Rprec_at (const EPI *epi, const REL_INFO *rel_info,
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
int te_calc_ndcg_at (const EPI *epi, const REL_INFO *rel_info,
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

#endif /* FUNCTIONSH */
