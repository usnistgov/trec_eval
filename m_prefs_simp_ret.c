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

/* Simple ratio of preferences fulfilled to preferences possible among the
   retrieved docs.
   If a doc pair satisfies two preferences, both are counted.
   If preferences are conflicted for a doc pair, all are counted
   (and thus max possible score may be less than 1.0 for topic).
   For doc pref A>B, A and B must both be retrieved to be counted as either
   fulfilled or possible.
 */

int 
te_calc_prefs_simp_ret (const EPI *epi, const REL_INFO *rel_info,
		     const RESULTS *results, const TREC_MEAS *tm,
		     TREC_EVAL *eval)
{
    RESULTS_PREFS results_prefs;
    long i;
    long poss, ful;

    if (UNDEF == form_prefs_counts (epi, rel_info, results, &results_prefs))
	return (UNDEF);
    
    ful = poss = 0;
    for (i = 0; i < results_prefs.num_jgs; i++) {
	ful += results_prefs.jgs[i].num_prefs_fulfilled_ret;
	poss += results_prefs.jgs[i].num_prefs_possible_ret;
    }
    /* Simple ratio of preferences fulfilled to preferences possible */
    if (poss) {
        eval->values[tm->eval_index].value =
	    (double) ful / (double) poss;
    }
    return (1);
}
