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

/*    "    Set Recall: num_relevant_retrieved / num_relevant \n\
    Recall over all docs retrieved for a topic.\n\
    Was known as exact_recall in earlier versions of trec_eval",
*/

int 
te_calc_set_recall (const EPI *epi, const REL_INFO *rel_info,
		    const RESULTS *results, const TREC_MEAS *tm,
		    TREC_EVAL *eval)
{
    RES_RELS res_rels;

    if (UNDEF == te_form_res_rels (epi, rel_info, results, &res_rels))
	return (UNDEF);

    eval->values[tm->eval_index].value =
	(double) res_rels.num_rel_ret /
	(double) res_rels.num_rel;

    return (1);
}
