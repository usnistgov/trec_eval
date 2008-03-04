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

/* Need comment and explanation */
int 
te_calc_num_q (const EPI *epi, const REL_INFO *rel_info,
		   const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    eval->values[tm->eval_index].value = 1;
    return (1);
}

/* Need custom calc_avg because after -c flag, calculated value may not
   agree with eval->num_queries */
int 
te_calc_avg_num_q (const EPI *epi, const TREC_MEAS *tm, TREC_EVAL *accum_eval)
{
    accum_eval->values[tm->eval_index].value = accum_eval->num_queries;
    return (1);
}    
