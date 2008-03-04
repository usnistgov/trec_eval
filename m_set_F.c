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

/*  Set F measure: weighted harmonic mean of recall and precision
    (x+1) * P * R / (R + x*P)
    where x is the relative importance of R to P (default 1.0).
    Variant of van Rijsbergen's E measure (Information Retrieval 1979).
*/

int 
te_calc_set_F (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	       const TREC_MEAS *tm, TREC_EVAL *eval)
{
    double *params = (double *) tm->meas_params->param_values;
    RES_RELS res_rels;
    double x, P, R;

    if (UNDEF == te_form_res_rels (epi, rel_info, results, &res_rels))
	return (UNDEF);

    if (res_rels.num_ret && res_rels.num_rel)  {
	P = (double) res_rels.num_rel_ret /
	    (double) res_rels.num_ret;
	R = (double) res_rels.num_rel_ret /
	    (double) res_rels.num_rel;
	x = params[0];
	eval->values[tm->eval_index].value =
	    (x + 1.0) * P * R / (x * P + R);
    }
    return (1);
}
