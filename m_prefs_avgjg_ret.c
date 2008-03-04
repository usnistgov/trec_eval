#ifdef RCSID
static char rcsid[] = "$Header: /home/smart/release/src/libevaluate/trvec_trec_eval.c,v 11.0 1992/07/21 18:20:35 chrisb Exp chrisb $";
#endif

/* Copyright (c) 2008
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "trec_format.h"

/* Simple ratio of preferences fulfilled to preferences possible, among the
   retrieved docs, within a judgment group, averaged over jgs.  
   I.e., rather than considering all preferences equal (prefs_simp),
   consider all judgment groups equal.
   prefs_avgjg = AVERAGE_OVER_JG (fulfilled_jg / possible_jg);
   May be useful in applications where user satisfaction is represented
   by a jg per user, and it is not desirable for many preferences expressed
   by user1 to swamp a few preferences by user2.
   For doc pref A>B, A and B must both be retrieved to be counted as either
   fulfilled or possible.
 */

int 
te_calc_prefs_avgjg_ret (const EPI *epi, const REL_INFO *rel_info,
			 const RESULTS *results, const TREC_MEAS *tm,
			 TREC_EVAL *eval)
{
    RESULTS_PREFS results_prefs;
    long i;
    long ful, poss;
    double sum;

    if (UNDEF == form_prefs_counts (epi, rel_info, results, &results_prefs))
	return (UNDEF);
    
    sum = 0.0;

    for (i = 0; i < results_prefs.num_jgs; i++) {
        ful = results_prefs.jgs[i].num_prefs_fulfilled_ret;
        poss = results_prefs.jgs[i].num_prefs_possible_ret;

	if (poss)
	    sum += (double) ful / (double) poss;
    }
    /* Simple ratio of preferences fulfilled to preferences possible in
     each jg, averaged over jgs */
    if (sum > 0.0) {
        eval->values[tm->eval_index].value =
	    sum / (double) results_prefs.num_jgs;
    }
    return (1);
}
