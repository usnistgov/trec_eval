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

double log2(double x);

/* Based on implementation by Ian Soboroff
   Compute a traditional nDCG measure according to Jarvelin and
   Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002).

   Gain values are set to the appropriate relevance level by default.  
   The default gain can be overridden on the command line by having 
   comma separated parameters 'rel_level=gain'.
   Eg, 'trec_eval -m ndcg_p.1=3.5,2=9.0,4=7.0 ...'
   will give gains 3.5, 9.0, 3.0, 7.0 for relevance levels 1,2,3,4
   respectively (level 3 remains at the default).
   Gains are allowed to be 0 or negative, and relevance level 0
   can be given a gain\n"
*/

/* Keep track of valid rel_levels and associated gains */
/* Initialized in setup_gains */
static struct rel_gain {
    long rel_level;
    long num_at_level;
    double gain;
} *gain_ptr = NULL;
static long num_gains = 0;
static long max_num_gains = 0;
static long total_num_at_levels = 0;

static int setup_gains (const TREC_MEAS *tm, const RANK_REL *rank_rel);
static double get_gain (long rel_level);
static int comp_rel_gain ();

int 
te_calc_ndcg_p (const EPI *epi, const REL_INFO *rel_info,
		const RESULTS *results, const TREC_MEAS *tm, TREC_EVAL *eval)
{
    RANK_REL rank_rel;
    double gain, sum;
    double ideal_dcg;          /* ideal discounted cumulative gain */
    long cur_lvl, lvl_count;
    long i;
   
    if (UNDEF == form_ordered_rel (epi, rel_info, results, &rank_rel))
	return (UNDEF);

    if (UNDEF == setup_gains (tm, &rank_rel))
	return (UNDEF);

    sum = 0.0;
    for (i = 0; i < rank_rel.num_ret; i++) {
	gain = get_gain (rank_rel.results_rel_list[i]);
	if (gain != 0) {
	    if (i > 0)
		sum += gain / log2((double) (i+1));
	    else
		sum += gain;
	    if (epi->debug_level > 0) 
		printf("ndcg_p:%ld %3.1f %6.4f\n", i, gain, sum);
	}
    }
    /* Calculate ideal discounted cumulative gain for this topic */
    cur_lvl = num_gains - 1;
    lvl_count = 0;
    ideal_dcg = 0.0;
    for (i = 0; i < total_num_at_levels; i++) {
	lvl_count++;
	while (lvl_count > gain_ptr[cur_lvl].num_at_level) {
	    lvl_count = 1;
	    cur_lvl--;
	    if (cur_lvl < 0 || gain_ptr[cur_lvl].gain <= 0.0)
		break;
	}
	if (cur_lvl < 0 || gain_ptr[cur_lvl].gain <= 0.0)
	    break;
	gain = gain_ptr[cur_lvl].gain;
	if (i == 0)
	    ideal_dcg += gain;
	else
	    ideal_dcg += gain / (float) log2((double)(i + 1));
	if (epi->debug_level > 0) 
	    printf("ndcg_p:%ld %ld %3.1f %6.4f\n", i, cur_lvl, gain, ideal_dcg);
    }

    /* Compare sum to ideal NDCG */
    if (rank_rel.num_rel_ret > 0) {
        eval->values[tm->eval_index].value =
	    sum / ideal_dcg;
    }
    return (1);
}

static int
setup_gains (const TREC_MEAS *tm, const RANK_REL *rank_rel)
{
    PAIR_PARAMS *params = (PAIR_PARAMS *) tm->meas_params;
    long num_params = (params ? params->num_params : 0);
    long i,j;

    if (rank_rel->num_rel_levels + num_params > max_num_gains) {
	/* Need more space (num_rel_levels may change on per query basis) */
	if (max_num_gains > 0)
	    Free (gain_ptr);
	max_num_gains += rank_rel->num_rel_levels + num_params;
	if (NULL == (gain_ptr = Malloc(max_num_gains, struct rel_gain)))
	    return (UNDEF);
	num_gains = 0;
	for (i = 0; i < num_params; i++) {
	    gain_ptr[num_gains].rel_level = atol (params->param_values[i].name);
	    gain_ptr[num_gains].gain = (double) params->param_values[i].value;
	    gain_ptr[num_gains].num_at_level = 0;
	    num_gains++;
	}
    }
    else {
	for (i = 0; i < max_num_gains; i++)
	    gain_ptr[i].num_at_level = 0;
    }

    for (i = 0; i < rank_rel->num_rel_levels; i++) {
	for (j = 0; j < num_gains && gain_ptr[j].rel_level != i; j++)
	    ;
	if (j < num_gains)
	    gain_ptr[j].num_at_level = rank_rel->rel_levels[i];
	else {
	    gain_ptr[num_gains].rel_level = i;
	    gain_ptr[num_gains].gain = (double) i;
	    gain_ptr[num_gains].num_at_level = rank_rel->rel_levels[i];
	    num_gains++;
	}
    }

    /* Sort gains by increasing gain value */
    qsort ((char *) gain_ptr,
           (int) num_gains,
           sizeof (struct rel_gain),
           comp_rel_gain);

    total_num_at_levels = 0;
    for (i = 0; i < num_gains; i++)
	total_num_at_levels += gain_ptr[i].num_at_level;	

    return (1);
}

static int comp_rel_gain (struct rel_gain *ptr1, struct rel_gain *ptr2)
{
    return (ptr1->gain - ptr2->gain);
}

static double
get_gain (long rel_level)
{
    long i;
    for (i = 0; i < num_gains; i++)
	if (rel_level == gain_ptr[i].rel_level)
	    return (gain_ptr[i].gain);
    return (0.0);   /* Print Error ?? */
}

