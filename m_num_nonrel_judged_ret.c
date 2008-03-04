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

/*      "    Number of non-relevant judged documents retrieved for topic. \n\
    Not an evaluation number per se, but gives details of retrieval results.\n\
    Summary figure is sum of individual topics, not average.\n",
*/

int 
te_calc_num_nonrel_judged_ret (const EPI *epi, const REL_INFO *rel_info,
			       const RESULTS *results,  const TREC_MEAS *tm,
			       TREC_EVAL *eval)
{
    RES_RELS res_rels;

    if (UNDEF == te_form_res_rels (epi, rel_info, results, &res_rels))
	return (UNDEF);

    eval->values[tm->eval_index].value = (double)
	res_rels.num_ret -
	res_rels.num_nonpool -
	res_rels.num_unjudged_in_pool -
	res_rels.num_rel_ret;

    return (1);
}
