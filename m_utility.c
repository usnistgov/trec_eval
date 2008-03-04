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

/*      "    Set utility measure\n\
    Set evaluation based on contingency table:\n\
                        relevant  nonrelevant\n\
       retrieved            a          b\n\
       nonretrieved         c          d\n\
    where  utility = p1 * a + p2 * b + p3 * c + p4 * d\n\
    and p1-4 are parameters (given on command line in that order).\n\
    Conceptually, each retrieved relevant doc is worth something positive to\n\
    a user, each retrieved nonrelevant doc has a negative worth, each \n\
    relevant doc not retrieved may have a negative worth, and each\n\
    nonrelevant doc not retrieved may have a (small) positive worth.\n\
    The overall measure is simply a weighted sum of these values.\n\
    If p4 is non-zero, then '-N num_docs_in_coll' may also be needed - the\n\
    standard results and rel_info files do not contain that information.\n\
    Default usage: trec_eval -m utility.1.0,-1.0,0.0,0.0 ...\n\
    Warning: Current version summary evaluation averages over all topics;\n\
    it could be argued that simply summing is more useful (but not backward\n\
    compatible)\n",
*/

int 
te_calc_utility (const EPI *epi, const REL_INFO *rel_info,
		 const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    FLOAT_PARAMS *params = (FLOAT_PARAMS *) tm->meas_params;
    RANK_REL rr;

    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rr))
	return (UNDEF);

    if (params->num_params != 4) {
	fprintf (stderr,
		 "trec_eval.calc_utility: improper number of coefficients\n");
	return (UNDEF);
    }

    eval->values[tm->eval_index].value =
	params->param_values[0] * rr.num_rel_ret +
	params->param_values[1] * (rr.num_ret - rr.num_rel_ret) +
	params->param_values[2] * (rr.num_rel - rr.num_rel_ret) +
	params->param_values[3] * (epi->num_docs_in_coll + rr.num_rel_ret -
				   rr.num_ret - rr.num_rel);
    return (1);
}
