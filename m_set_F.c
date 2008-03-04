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

/*  Set F measure: weighted harmonic mean of recall and precision
    (x+1) * P * R / (R + x*P)
    where x is the relative importance of R to P (default 1.0).
    Variant of van Rijsbergen's E measure (Information Retrieval 1979).
*/

int 
te_calc_set_F (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	       const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;
    double x, P, R;
    FLOAT_PARAMS *params = (FLOAT_PARAMS *) tm->meas_params;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    if (rank_rel.num_ret && rank_rel.num_rel)  {
	P = (double) rank_rel.num_rel_ret /
	    (double) rank_rel.num_ret;
	R = (double) rank_rel.num_rel_ret /
	    (double) rank_rel.num_rel;
	x = params->param_values[0];
	eval->values[tm->eval_index].value =
	    (x + 1.0) * P * R / (x * P + R);
    }
    return (1);
}
