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

/*     "    Main binary preference measure.\n\
    Fraction of the top R nonrelevant docs that are retrieved after each\n\
    relevant doc. Put another way: when looking at the R relevant docs, and\n\
    the top R nonrelevant docs, if all relevant docs are to be preferred to\n\
    nonrelevant docs, bpref is the fraction of the preferences that the\n\
    ranking preserves\n\
    Cite: 'Retrieval Evaluation with Incomplete Information', Chris Buckley\n\
    and Ellen Voorhees. In Proceedings of 27th SIGIR, 2004.\n",
*/
/*  Calculate an entire set of different bpref measures.  All of the measures are
   calculated at the same time and cached.  Only those measures that
   were requested are stored in eval.
   Note many of these measures may not even be in measures.c.
*/

static struct temp_eval {
    /* Binary Pref relations: fraction of nonrel documents retrieved after 
       each rel doc */
    double bpref;                     /* real BPREF.  Top num_rel nonrel docs */
    double gm_bpref;                  /* real BPREF, but geo,etric mean will be
					taken instead of average */
    double inf_ap;                    /* Inferred AP.  Not a bpref measure,
					but easily calculated here.  Take
					it out of here at some point */
    double bpref_top5Rnonrel;         /* Top 5 * num_rel nonrel docs */
    double bpref_top10Rnonrel;        /* Top 10 * num_rel nonrel docs */
/*    double bpref_topRnonrel;         *  renamed as bpref */
    double bpref_allnonrel;           /* all judged nonrel docs */
    double bpref_retnonrel;           /* Only retrieved nonrel docs */
    double bpref_topnonrel;           /* Top PREF_TOPNREL_NUM nonrel docs */
    double bpref_top50pRnonrel;       /* Top 50 + num_rel nonrel docs */
    double bpref_top25pRnonrel;       /* Top 25 + num_rel nonrel docs */
    double bpref_top10pRnonrel;       /* Top 10 + num_rel nonrel docs.
                                        Bad version used in SIGIR 2004 paper */
    double old_bpref_top10pRnonrel;   /* bad old version. Top 10 + num_rel 
                                        nonrel docs. Used in SIGIR 2004 paper*/
    double bpref_top25p2Rnonrel;      /* Top 25 + 2 * num_rel nonrel docs */
    double bpref_retall;              /* Only retrieved rel,nonrel docs */
    double bpref_5;                   /* Only top 5 rel, top 5 nonrel */
    double bpref_10;                  /* Only top 10 rel, top 10 nonrel */
    double old_bpref;                 /* Bad old bpref. Top num_rel nonrel docs.
                                        Only used retrieved nonrel docs.
                                        Used in TREC 12,13, mention in 
                                        SIGIR 2004 paper */
    long  bpref_num_all;             /* num not retrieved before (all judged)*/
    long  bpref_num_ret;             /* num retrieved after */
    long  bpref_num_correct;         /* num correct preferences */
    long  bpref_num_possible;        /* num possible correct preferences */

} temp_eval;

typedef struct {
    char *measure_name;
    double *float_ptr;
    long *long_ptr;
    long is_float_flag;
} NAME_VALUE;

static NAME_VALUE name_value[] = {
    {"bpref", &temp_eval.bpref, NULL, 1},
    {"gm_bpref", &temp_eval.gm_bpref, NULL, 1},
    {"infAP", &temp_eval.inf_ap, NULL, 1},
    {"bpref_top5Rnonrel", &temp_eval.bpref_top5Rnonrel, NULL, 1},
    {"bpref_top10Rnonrel", &temp_eval.bpref_top10Rnonrel, NULL, 1},
    {"bpref_allnonrel", &temp_eval.bpref_allnonrel, NULL, 1},
    {"bpref_retnonrel", &temp_eval.bpref_retnonrel, NULL, 1},
    {"bpref_topnonrel", &temp_eval.bpref_topnonrel, NULL, 1},
    {"bpref_top50pRnonrel", &temp_eval.bpref_top50pRnonrel, NULL, 1},
    {"bpref_top25pRnonrel", &temp_eval.bpref_top25pRnonrel, NULL, 1},
    {"bpref_top10pRnonrel", &temp_eval.bpref_top10pRnonrel, NULL, 1},
    {"old_bpref_top10pRnonrel", &temp_eval.old_bpref_top10pRnonrel, NULL, 1},
    {"bpref_top25p2Rnonrel", &temp_eval.bpref_top25p2Rnonrel, NULL, 1},
    {"bpref_retall", &temp_eval.bpref_retall, NULL, 1},
    {"bpref_5", &temp_eval.bpref_5, NULL, 1},
    {"bpref_10", &temp_eval.bpref_10, NULL, 1},
    {"old_bpref", &temp_eval.old_bpref, NULL, 1},
    {"bpref_num_all", NULL, &temp_eval.bpref_num_all, 0},
    {"bpref_num_ret", NULL, &temp_eval.bpref_num_ret, 0},
    {"bpref_num_correct", NULL, &temp_eval.bpref_num_correct, 0},
    {"bpref_num_possible", NULL, &temp_eval.bpref_num_possible, 0},
};

static long num_name_value = sizeof (name_value)/sizeof (name_value[0]);

/* Current cached query */
static char current_query[MAX_LEN_QUERY] = "no query";

static int calc_temp_eval (const EPI *epi, RANK_REL *rank_rel);

int 
te_calc_bpref (const EPI *epi, const REL_INFO *rel_info, const RESULTS *results,
	       const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;
    long i;
    if (strncmp (current_query, results->qid, MAX_LEN_QUERY)) {
	/* New query, must recompute cached temp_eval measures */
	if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	    return (UNDEF);
	if (UNDEF == calc_temp_eval (epi, &rank_rel))
	    return (UNDEF);

	(void) strncpy (current_query, results->qid, MAX_LEN_QUERY);
    }

    /* Value has been calculated in temp_eval, find it and store in eval */
    for (i = 0; i < num_name_value; i++) {
	if (0 == strcmp (tm->name, name_value[i].measure_name))
	    break;
    }
    if (i >= num_name_value)
	return (UNDEF);

    if (name_value[i].is_float_flag)
        eval->values[tm->eval_index].value =
	    *name_value[i].float_ptr;
    else
        eval->values[tm->eval_index].value = (double)
	    *name_value[i].long_ptr;
    return (1);
}

/* Set a maximum number of nonrel docs to be used for preference measures */
#define PREF_TOP_NONREL_NUM 100

/* Code for calc_temp_eval taken directly from trvec_teval in
   trec_eval version 8.2.  Messy enough so I don't want to recode! */

static int
calc_temp_eval (const EPI *epi, RANK_REL *rank_rel)
{
    long j;
    long nonrel_ret;
    long nonrel_so_far, rel_so_far, pool_unjudged_so_far;
    long bounded_5R_nonrel_so_far, bounded_10R_nonrel_so_far;
    long pref_top_nonrel_num = PREF_TOP_NONREL_NUM;
    long pref_top_50pRnonrel_num;
    long pref_top_25pRnonrel_num;
    long pref_top_25p2Rnonrel_num;
    long pref_top_10pRnonrel_num;
    long pref_top_Rnonrel_num;

    long num_rel = rank_rel->num_rel;
    long num_nonrel = 0;

    for (j = 0; j < rank_rel->num_rel_levels; j++)
	num_nonrel += rank_rel->rel_levels[j];
    num_nonrel = num_nonrel - num_rel;

    /* Zero out all values in temp_eval */
    (void) memset ((void *) &temp_eval, 0, sizeof (temp_eval));

    /* Calculate judgement based measures (dependent on only
       judged docs; no assumption of non-relevance if not judged) */
    /* Binary Preference measures; here expressed as all docs with a higher 
       value of rel are to be preferred.  Optimize by keeping track of nonrel
       seen so far */
    pref_top_nonrel_num = PREF_TOP_NONREL_NUM;
    pref_top_50pRnonrel_num = 50 + num_rel;
    pref_top_25pRnonrel_num = 25 + num_rel;
    pref_top_10pRnonrel_num = 10 + num_rel;
    pref_top_Rnonrel_num = num_rel;
    pref_top_25p2Rnonrel_num = 25 + (2 * num_rel);
    nonrel_ret = rank_rel->num_ret - 
	rank_rel->num_nonpool -
	rank_rel->num_unjudged_in_pool -
	rank_rel->num_rel_ret;

    nonrel_so_far = 0;
    rel_so_far = 0;
    pool_unjudged_so_far = 0;
    bounded_5R_nonrel_so_far = 0; 
    bounded_10R_nonrel_so_far = 0; 
    for (j = 0; j < rank_rel->num_ret; j++) {
	if (rank_rel->results_rel_list[j] == RELVALUE_NONPOOL)
	    /* document not in pool. Skip */
	    continue;
	if (rank_rel->results_rel_list[j] == RELVALUE_UNJUDGED) {
	    /* document in pool but unjudged. */
	    pool_unjudged_so_far++;
	    continue;
	}

	if (rank_rel->results_rel_list[j] >= 0 && rank_rel->results_rel_list[j] < epi->relevance_level) {
	    /* Judged Nonrel document */
	    if (nonrel_so_far < 5 * num_rel) {
		bounded_5R_nonrel_so_far++;
		if (nonrel_so_far < 10 * num_rel) {
		    bounded_10R_nonrel_so_far++;
		}
	    }
	    nonrel_so_far++;
	}
	else {
	    /* Judged Rel doc */
	    rel_so_far++;
	    /* Add fraction of correct preferences. */
	    /* Special case nonrel_so_far == 0 to avoid division by 0 */
	    if (nonrel_so_far > 0) {
		temp_eval.bpref_allnonrel += 1.0 - (((double) nonrel_so_far) /
					       (double) num_nonrel);
		temp_eval.bpref_retnonrel += 1.0 - (((double) nonrel_so_far) /
					       (double) nonrel_ret);
		temp_eval.bpref_retall += 1.0 - (((double) nonrel_so_far) /
					    (double) nonrel_ret);
		temp_eval.bpref_num_correct += 
		    MIN (num_nonrel, pref_top_Rnonrel_num) -
		    MIN (nonrel_so_far, pref_top_Rnonrel_num);
		temp_eval.bpref += 1.0 - 
		    (((double) MIN (nonrel_so_far, pref_top_Rnonrel_num)) /
		     (double) MIN (num_nonrel, pref_top_Rnonrel_num));
		temp_eval.old_bpref += 1.0 - 
		    (((double) MIN (nonrel_so_far, pref_top_Rnonrel_num)) /
		     (double) MIN (nonrel_ret, pref_top_Rnonrel_num));
		temp_eval.bpref_topnonrel += 1.0 - 
		    (((double) MIN (nonrel_so_far, pref_top_nonrel_num)) /
		     (double) MIN (num_nonrel, pref_top_nonrel_num));
		temp_eval.bpref_top50pRnonrel += 1.0 - 
		    (((double) MIN (nonrel_so_far, pref_top_50pRnonrel_num)) /
		     (double) MIN (num_nonrel, pref_top_50pRnonrel_num));
		temp_eval.bpref_top25pRnonrel += 1.0 - 
		    (((double) MIN (nonrel_so_far, pref_top_25pRnonrel_num)) /
		     (double) MIN (num_nonrel, pref_top_25pRnonrel_num));
		temp_eval.bpref_top10pRnonrel += 1.0 - 
		    (((double) MIN (nonrel_so_far, pref_top_10pRnonrel_num)) /
		     (double) MIN (num_nonrel, pref_top_10pRnonrel_num));
		temp_eval.old_bpref_top10pRnonrel += 1.0 - 
		    (((double) MIN (nonrel_so_far, pref_top_10pRnonrel_num)) /
		     (double) MIN (nonrel_ret, pref_top_10pRnonrel_num));
		temp_eval.bpref_top25p2Rnonrel += 1.0 - 
		    (((double) MIN (nonrel_so_far, pref_top_25p2Rnonrel_num)) /
		     (double) MIN (num_nonrel, pref_top_25p2Rnonrel_num));
		if (rel_so_far <= 5 && nonrel_so_far < 5)
		    temp_eval.bpref_5 += 1.0 - (double) nonrel_so_far /
			(double) MIN (num_nonrel, 5);
		if (rel_so_far <= 10 && nonrel_so_far < 10)
		    temp_eval.bpref_10 += 1.0 - (double) nonrel_so_far /
			(double) MIN (num_nonrel, 10);
	    }
	    else {
		temp_eval.bpref += 1.0;
		temp_eval.old_bpref += 1.0;
		temp_eval.bpref_allnonrel += 1.0;
		temp_eval.bpref_retnonrel += 1.0;
		temp_eval.bpref_retall += 1.0;
		temp_eval.bpref_topnonrel += 1.0;
		temp_eval.bpref_top50pRnonrel += 1.0;
		temp_eval.bpref_top25pRnonrel += 1.0;
		temp_eval.bpref_top10pRnonrel += 1.0;
		temp_eval.old_bpref_top10pRnonrel += 1.0;
		temp_eval.bpref_top25p2Rnonrel += 1.0;
		if (rel_so_far <= 5)
		    temp_eval.bpref_5 += 1.0;
		if (rel_so_far <= 10)
		    temp_eval.bpref_10 += 1.0;

	    }
	    temp_eval.bpref_top5Rnonrel += 1.0 -
		(((double) bounded_5R_nonrel_so_far) /
		 (double) MIN (num_nonrel, num_rel * 5));
	    temp_eval.bpref_top10Rnonrel += 1.0 -
		(((double) bounded_10R_nonrel_so_far) /
		 (double) MIN (num_nonrel, num_rel * 10));
	    temp_eval.bpref_num_all += num_nonrel - nonrel_so_far;
	    temp_eval.bpref_num_ret += nonrel_ret - nonrel_so_far;
	    /* inf_ap */
	    if (0 == j)
		temp_eval.inf_ap += 1.0;
	    else {
		double fj = (double) j;
		temp_eval.inf_ap += 1.0 / (fj+1.0) +
		    (fj / (fj+1.0)) *
		    ((rel_so_far-1+nonrel_so_far+pool_unjudged_so_far) / fj)  *
		    ((rel_so_far-1 + INFAP_EPSILON) / 
		     (rel_so_far-1 + nonrel_so_far + 2 * INFAP_EPSILON));
	    }
	}
    }
    if (num_rel) {
	temp_eval.bpref /= num_rel;
	temp_eval.old_bpref /= num_rel;
	temp_eval.bpref_allnonrel /= num_rel;
	temp_eval.bpref_retnonrel /= num_rel;
	temp_eval.bpref_topnonrel /= num_rel;
	temp_eval.bpref_top5Rnonrel /= num_rel;
	temp_eval.bpref_top10Rnonrel /= num_rel;
	temp_eval.bpref_top50pRnonrel /= num_rel;
	temp_eval.bpref_top25pRnonrel /= num_rel;
	temp_eval.bpref_top10pRnonrel /= num_rel;
	temp_eval.old_bpref_top10pRnonrel /= num_rel;
	temp_eval.bpref_top25p2Rnonrel /= num_rel;
	if (rank_rel->num_rel_ret) {
	    temp_eval.bpref_retall /= rank_rel->num_rel_ret;
	    temp_eval.bpref_5 /= MIN (rel_so_far, 5);
	    temp_eval.bpref_10 /= MIN (rel_so_far, 10);
	}
	temp_eval.bpref_num_possible = num_rel *
	    MIN (num_nonrel, pref_top_Rnonrel_num);
	temp_eval.inf_ap /= num_rel;
    }
    /*    temp_eval.num_nonrel_judged_ret = nonrel_ret; */

    /* For those bpref measure variants which use the geometric mean instead
       of straight averages, compute them here.  Original measure value
       is constrained to be greater than MIN_GEO_MEAN (for time being .00001,
       since trec_eval prints to four significant digits) */
    temp_eval.gm_bpref = (double) log ((double)(MAX (temp_eval.bpref,
						MIN_GEO_MEAN)));

    return (1);
}
