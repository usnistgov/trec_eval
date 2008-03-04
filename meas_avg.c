/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"

/* Generic procedures for calculating averages of measures, see trec_eval.h
      typedef struct trec_meas {
     ... 
     * Calculate final averages (if needed)  from summary info *
     int (* calc_average) (const EPI *epi, const struct trec_meas *tm,
                           const TREC_EVAL *eval);
     ...
      } TREC_MEAS;
   Measures are defined in measures.c.
*/

/* Measure does not require averaging */
int
te_calc_avg_meas_empty (const EPI *epi, const TREC_MEAS *tm,
			TREC_EVAL *eval)
{
    return (1);
}

/* Measure is a single float/long that should now be averaged */
int
te_calc_avg_meas_s (const EPI *epi, const TREC_MEAS *tm,
		    TREC_EVAL *accum_eval)
{
    if (accum_eval->num_queries)
	accum_eval->values[tm->eval_index].value /= accum_eval->num_queries;
    return (1);
}

/* Measure is an array with cutoffs */
int
te_calc_avg_meas_a_cut (const EPI *epi, const TREC_MEAS *tm,
			     TREC_EVAL *accum_eval)
{
    long i;
    
    if (accum_eval->num_queries) {
	for (i = 0; i < tm->meas_params->num_params; i++) {
	    accum_eval->values[tm->eval_index + i].value /=
		accum_eval->num_queries;
	}
    }
    return (1);
}

/* Measure is a single float with no parameters to be averaged with
   geometric mean */
int
te_calc_avg_meas_s_gm (const EPI *epi, const TREC_MEAS *tm,
		       TREC_EVAL *accum_eval)
{
    double sum;

    if (accum_eval->num_queries > 0) {
	sum  = accum_eval->values[tm->eval_index].value;
	if (epi->average_complete_flag)
	    /* Must patch up averages for any missing queries, since */
	    /* value of 0 means perfection */
	    sum += (accum_eval->num_queries - accum_eval->num_orig_queries) *
		log (MIN_GEO_MEAN);

	accum_eval->values[tm->eval_index].value =
	    exp ((double) (sum / accum_eval->num_queries));
    }
    return (1);
}












