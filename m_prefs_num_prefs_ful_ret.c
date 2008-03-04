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

/* Number of prefs fulfilled
     For doc pref A>B, both A nd B must be retrieved to be counted.
    Summary figure is sum of individual topics, not average.
 */

int 
te_calc_prefs_num_prefs_ful_ret (const EPI *epi, const REL_INFO *rel_info,
				 const RESULTS *results, const TREC_MEAS *tm,
				 TREC_EVAL *eval)
{
    RESULTS_PREFS results_prefs;
    long i;
    long ful;

    if (UNDEF == form_prefs_counts (epi, rel_info, results, &results_prefs))
	return (UNDEF);
    
    ful = 0;
    for (i = 0; i < results_prefs.num_jgs; i++) {
	ful += results_prefs.jgs[i].num_prefs_fulfilled_ret;
    }
    eval->values[tm->eval_index].value = (double) ful;
    return (1);
}
