#ifdef RCSID
static char rcsid[] = "$Header: /home/smart/release/src/libevaluate/trvec_trec_eval.c,v 11.0 1992/07/21 18:20:35 chrisb Exp chrisb $";
#endif

/* Copyright (c) 1991, 1990, 1984 - Gerard Salton, Chris Buckley. 

   Permission is granted for use of this file in unmodified form for
   research purposes. Please contact the SMART project to obtain 
   permission for other uses.
*/

/********************   PROCEDURE DESCRIPTION   ************************
 *0 Given ranks for top docs for a query, evaluate the query
 *2 trvec_trec_eval (tr_vec, eval, inst)
 *3   TR_VEC *tr_vec;
 *3   TREC_EVAL *eval;
 *3   int inst;
 *4 init_trvec_trec_eval (spec, unused)
 *5   "eval.qrels_file"
 *5   "eval.qrels_file.rmode"
 *5   "trvec_trec_eval.trace"
 *4 close_trvec_trec_eval (inst)

 *7 Eval is of type (from trec_eval.h) 
 *7 WARNING: Comment needs to be updated to include new measures (eg bpref).
 *7 * Defined constants that are collection/purpose dependent *
 *7 
 *7 * Number of cutoffs for recall,precision, and rel_precis measures. *
 *7 * CUTOFF_VALUES gives the number of retrieved docs that these *
 *7 * evaluation mesures are applied at. *
 *7 #define NUM_CUTOFF  6
 *7 #define CUTOFF_VALUES  {5, 15, 30, 100, 200, 500}
 *7 
 *7 * Maximum fallout value, expressed in number of non-rel docs retrieved. *
 *7 * (Make the approximation that number of non-rel docs in collection *
 *7 * is equal to the number of number of docs in collection) *
 *7 #define MAX_FALL_RET  142
 *7 
 *7 * Maximum multiple of R (number of rel docs for this query) to calculate *
 *7 * R-based precision at *
 *7 #define MAX_RPREC 2.0
 *7 
 *7 
 *7 * ----------------------------------------------- *
 *7 * Defined constants that are collection/purpose independent.  If you
 *7    change these, you probably need to change comments and documentation,
 *7    and some variable names may not be appropriate any more! *
 *7 #define NUM_RP_PTS  11
 *7 #define THREE_PTS {2, 5, 8}    
 *7 #define NUM_FR_PTS  11
 *7 #define NUM_PREC_PTS 11
 *7 
 *7 typedef struct {
 *7     long  qid;                      * query id  (for overall average figures,
 *7                                        this gives number of queries in run) *
 *7     * Summary Numbers over all queries *
 *7     long num_rel;                   * Number of relevant docs *
 *7     long num_ret;                   * Number of retrieved docs *
 *7     long num_rel_ret;               * Number of relevant retrieved docs *
 *7 
 *7     * Measures after num_ret docs *
 *7     float exact_recall;             * Recall after num_ret docs *
 *7     float exact_precis;             * Precision after num_ret docs *
 *7     float exact_rel_precis;         * Relative Precision (or recall) *
 *7                                     * Defined to be precision  max possible
 *7                                        precision *
 *7 
 *7     * Measures after each document *
 *7     float recall_cut[NUM_CUTOFF];   * Recall after cutoff[i] docs *
 *7 
 *7     float precis_cut[NUM_CUTOFF];   * precision after cutoff[i] docs. If
 *7                                        less than cutoff[i] docs retrieved,
 *7                                        then assume an additional 
 *7                                        cutoff[i]-num_ret non-relevant docs
 *7                                        are retrieved. *
 *7     float rel_precis_cut[NUM_CUTOFF];* Relative precision after cutoff[i] 
 *7                                        docs.   *
 *7 
 *7     * Measures after each rel doc *
 *7     float av_recall_precis;         * average(integral) of precision at
 *7                                        all rel doc ranks *
 *7     float int_av_recall_precis;     * Same as above, but the precision values
 *7                                        have been interpolated, so that prec(X)
 *7                                        is actually MAX prec(Y) for all 
 *7                                        Y >= X   *
 *7     float int_recall_precis[NUM_RP_PTS];* interpolated precision at 
 *7                                        0.1 increments of recall *
 *7     float int_av3_recall_precis;    *interpolated average at 3 intermediate
 *7                                        points *
 *7     float int_av11_recall_precis;   * interpolated average at NUM_RP_PTS 
 *7                                        intermediate points (recall_level) *
 *7 
 *7     * Measures after each non-rel doc *
 *7     float fall_recall[NUM_FR_PTS];  * max recall after each non-rel doc,
 *7                                        at 11 points starting at 0.0 and
 *7                                        ending at MAX_FALL_RET / num_docs *
 *7     float av_fall_recall;           * Average of fallout-recall, after each
 *7                                        non-rel doc until fallout of 
 *7                                        MAX_FALL_RET / num_docs achieved *
 *7 
 *7     * Measures after R-related cutoffs.  R is the number of relevant
 *7      docs for a particular query, but note that these cutoffs are after
 *7      R docs, whether relevant or non-relevant, have been retrieved.
 *7      R-related cutoffs are really only applicable to a situtation where
 *7      there are many relevant docs per query (or lots of queries). *
 *7     float R_recall_precis;          * Recall or precision after R docs
 *7                                       (note they are equal at this point) *
 *7     float av_R_precis;              * Average (or integral) of precision at
 *7                                        each doc until R docs have been 
 *7                                        retrieved *
 *7     float R_prec_cut[NUM_PREC_PTS]; * Precision measured after multiples of
 *7                                        R docs have been retrieved.  11 
 *7                                        equal points, with max multiple
 *7                                        having value MAX_RPREC *
 *7     float int_R_recall_precis;      * Interpolated precision after R docs
 *7                                       Prec(X) = MAX(prec(Y)) for all Y>=X *
 *7     float int_av_R_precis;          * Interpolated *
 *7     float int_R_prec_cut[NUM_PREC_PTS]; * Interpolated *
***********************************************************************/

#include "common.h"
#include "sysfunc.h"
#include "smart_error.h"
#include "tr_vec.h"
#include "trec_eval.h"

 /* cutoff values for recall precision output */
static int cutoff[NUM_CUTOFF] = CUTOFF_VALUES;
static int three_pts[3] = THREE_PTS;

static int compare_iter_rank();

int
trvec_trec_eval (epi, tr_vec, eval, num_rel, num_nonrel)
EVAL_PARAM_INFO *epi;
TR_VEC *tr_vec;
TREC_EVAL *eval;
long num_rel;               /* Number relevant judged */
long num_nonrel;            /* Number nonrelevant judged */
{
    double recall, precis;     /* current recall, precision values */
    double rel_precis, rel_uap;/* relative precision, uap values */
    double int_precis;         /* current interpolated precision values */
    
    long i,j;
    long rel_so_far, nonrel_so_far, nonrel_ret;
    long max_iter;

    long cut_rp[NUM_RP_PTS];   /* number of rel docs needed to be retrieved
                                  for each recall-prec cutoff */
    long cut_fr[NUM_FR_PTS];   /* number of non-rel docs needed to be
                                  retrieved for each fall-recall cutoff */
    long cut_rprec[NUM_PREC_PTS]; /* Number of docs needed to be retrieved
                                    for each R-based prec cutoff */
    long current_cutoff, current_cut_rp, current_cut_fr, current_cut_rprec;

    long last_time_bucket = NUM_TIME_PTS;  /* Last time bucket filled in */

    long min_ret_rel;

    long pref_top_nonrel_num = PREF_TOP_NONREL_NUM;
    long pref_top_50pRnonrel_num;
    long pref_top_25pRnonrel_num;
    long pref_top_25p2Rnonrel_num;
    long pref_top_10pRnonrel_num;
    long pref_top_Rnonrel_num;
    long bounded_5R_nonrel_so_far, bounded_10R_nonrel_so_far;
    
    if (tr_vec == (TR_VEC *) NULL)
        return (UNDEF);

    /* Initialize everything to 0 */
    bzero ((char *) eval, sizeof (TREC_EVAL));

    eval->qid = tr_vec->qid;
    eval->num_queries = 1;

    /* If no retrieved docs, then just return */
    if (tr_vec->num_tr == 0) {
        return (0);
    }

    eval->num_rel = num_rel;

    /* Evaluate only the docs on the last iteration of new_tr_vec */
    /* Sort the tr tuples for this query by decreasing iter and 
       increasing rank */
    qsort ((char *) tr_vec->tr,
           (int) tr_vec->num_tr,
           sizeof (TR_TUP),
           compare_iter_rank);

    max_iter = tr_vec->tr[0].iter;
    rel_so_far = 0;
    for (j = 0; j < tr_vec->num_tr; j++) {
        if (tr_vec->tr[j].iter == max_iter) {
            eval->num_ret++;
            if (tr_vec->tr[j].rel >= epi->relevance_level)
                rel_so_far++;
        }
        else {
            if (tr_vec->tr[j].rel >= epi->relevance_level)
                eval->num_rel--;
        }
    }
    eval->num_rel_ret = rel_so_far;

    /* Discover cutoff values for this query */
    current_cutoff = NUM_CUTOFF - 1;
    while (current_cutoff > 0 && cutoff[current_cutoff] > eval->num_ret)
        current_cutoff--;
    for (i = 0; i < NUM_RP_PTS; i++)
        cut_rp[i] = ((eval->num_rel * i) + NUM_RP_PTS - 2) / (NUM_RP_PTS - 1);
    current_cut_rp = NUM_RP_PTS - 1;
    while (current_cut_rp > 0 && cut_rp[current_cut_rp] > eval->num_rel_ret)
        current_cut_rp--;
    for (i = 0; i < NUM_FR_PTS; i++)
        cut_fr[i] = ((MAX_FALL_RET * i) + NUM_FR_PTS - 2) / (NUM_FR_PTS - 1);
    current_cut_fr = NUM_FR_PTS - 1;
    while (current_cut_fr > 0 && cut_fr[current_cut_fr] > eval->num_ret - eval->num_rel_ret)
        current_cut_fr--;
    for (i = 1; i < NUM_PREC_PTS+1; i++)
        cut_rprec[i-1] = ((MAX_RPREC * eval->num_rel * i) + NUM_PREC_PTS - 2) 
                        / (NUM_PREC_PTS - 1);
    current_cut_rprec = NUM_PREC_PTS - 1;
    while (current_cut_rprec > 0 && cut_rprec[current_cut_rprec]>eval->num_ret)
        current_cut_rprec--;

    /* Note for interpolated precision values (Prec(X) = MAX (PREC(Y)) for all
       Y >= X) */
    int_precis = (float) rel_so_far / (float) eval->num_ret;

    min_ret_rel = MIN(eval->num_rel, eval->num_ret);

    /* Loop over all retrieved docs in reverse order */
    for (j = eval->num_ret; j > 0; j--) {
	if (rel_so_far > 0) {
	    recall = (float) rel_so_far / (float) eval->num_rel;
	    precis = (float) rel_so_far / (float) j;
	    if (j > eval->num_rel) {
		rel_precis = (float) rel_so_far / (float) eval->num_rel;
	    }
	    else {
		rel_precis = (float) rel_so_far / (float) j;
	    }
	}
	else {
	    recall = 0.0;
	    precis = 0.0;
	    rel_precis = 0.0;
	}
	rel_uap = rel_precis * rel_precis;
        if (int_precis < precis)
            int_precis = precis;
        while (j == cutoff[current_cutoff]) {
            eval->recall_cut[current_cutoff] = recall;
            eval->precis_cut[current_cutoff] = precis;
            eval->rel_precis_cut[current_cutoff] = rel_precis;
            eval->uap_cut[current_cutoff] = precis * recall;
	    eval->rel_uap_cut[current_cutoff] = rel_uap;
            current_cutoff--;
        }
	eval->av_rel_precis += rel_precis;
	eval->av_rel_uap += rel_uap;

        while (j == cut_rprec[current_cut_rprec]) {
            eval->R_prec_cut[current_cut_rprec] = precis;
            eval->int_R_prec_cut[current_cut_rprec] = int_precis;
            current_cut_rprec--;
        }

        if (j == eval->num_rel) {
            eval->R_recall_precis = precis;
            eval->int_R_recall_precis = int_precis;
        }

        if (j < eval->num_rel) {
            eval->av_R_precis += precis;
            eval->int_av_R_precis += int_precis;
        }

	if (epi->time_flag) {
	    long bucket = tr_vec->tr[j-1].sim *
		((double) NUM_TIME_PTS / (double) MAX_TIME);
	    if (bucket < 0) bucket = 0;
	    if (bucket >= NUM_TIME_PTS) bucket = NUM_TIME_PTS-1;
	    if (tr_vec->tr[j-1].rel >= epi->relevance_level)
		eval->time_num_rel[bucket]++;
	    else
		eval->time_num_nrel[bucket]++;
	    eval->time_precis[bucket] = (float)rel_so_far /
		(float) eval->num_ret;
	    eval->time_relprecis[bucket] = ((float)rel_so_far) / 
		(float) min_ret_rel;
	    eval->time_uap[bucket] = (float) rel_so_far * rel_so_far /
		((float) eval->num_ret * (float) min_ret_rel);
	    eval->time_reluap[bucket] = (float) rel_so_far * rel_so_far /
		((float) min_ret_rel * (float) min_ret_rel);
	    eval->time_utility[bucket] = 
		epi->utility_a * rel_so_far +
		epi->utility_b * (j - rel_so_far) +
		epi->utility_c * (eval->num_rel - rel_so_far) +
		epi->utility_d * (epi->num_docs_in_coll +
				 rel_so_far - j - eval->num_rel);

	    /* Need to fill in buckets up to last bucket */
	    /* note assumes buckets are decreasing */
	    /* Must do here since utility can be negative and zero
	       cannot be used as flag later */
	    for (i = bucket+1; i < last_time_bucket; i++) {
		eval->time_precis[i] = eval->time_precis[bucket];
		eval->time_relprecis[i] = eval->time_relprecis[bucket];
		eval->time_uap[i] = eval->time_uap[bucket];
		eval->time_reluap[i] = eval->time_reluap[bucket];
		eval->time_utility[i] = eval->time_utility[bucket];
	    }
	    last_time_bucket = bucket;
	}

        if (tr_vec->tr[j-1].rel >= epi->relevance_level) {
            eval->int_av_recall_precis += int_precis;
            eval->av_recall_precis += precis;
            eval->avg_doc_prec += precis;
            while (rel_so_far == cut_rp[current_cut_rp]) {
                eval->int_recall_precis[current_cut_rp] = int_precis;
                current_cut_rp--;
            }
	    eval->recip_rank = 1.0 / (float) j;
            rel_so_far--;
        }
        else {
            /* Note: for fallout-recall, the recall at X non-rel docs
               is used for the recall 'after' (X-1) non-rel docs.
               Ie. recall_used(X-1 non-rel docs) = MAX (recall(Y)) for 
               Y retrieved docs where X-1 non-rel retrieved */
            while (current_cut_fr >= 0 &&
                   j - rel_so_far == cut_fr[current_cut_fr] + 1) {
                eval->fall_recall[current_cut_fr] = recall;
                current_cut_fr--;
            }
            if (j - rel_so_far < MAX_FALL_RET) {
                eval->av_fall_recall += recall;
            }
        }
    }

    /* Fill in the 0.0 value for recall-precision (== max precision
       at any point in the retrieval ranking) */
    eval->int_recall_precis[0] = int_precis;

    /* Fill in those cutoff values and averages that were not achieved
       because insufficient docs were retrieved. */
    for (i = 0; i < NUM_CUTOFF; i++) {
        if (eval->num_ret < cutoff[i]) {
	    if (eval->num_rel_ret > 0) {
		eval->recall_cut[i] = ((float) eval->num_rel_ret /
				       (float) eval->num_rel);
		eval->precis_cut[i] = ((float) eval->num_rel_ret / 
				       (float) cutoff[i]);
	    }
            eval->rel_precis_cut[i] = (cutoff[i] < eval->num_rel) ?
                                            eval->precis_cut[i] :
                                            eval->recall_cut[i];
            eval->uap_cut[i] = eval->precis_cut[i] *
		                         eval->recall_cut[i];
            eval->rel_uap_cut[i] = eval->precis_cut[i] *
		                         eval->precis_cut[i];
	}
    }
    for (i = 0; i < NUM_FR_PTS; i++) {
        if (eval->num_ret - eval->num_rel_ret < cut_fr[i]) {
	    if (eval->num_rel_ret > 0)
		eval->fall_recall[i] = (float) eval->num_rel_ret / 
		                       (float) eval->num_rel;
        }
    }
    if (eval->num_ret - eval->num_rel_ret < MAX_FALL_RET) {
	if (eval->num_rel_ret > 0)
	    eval->av_fall_recall += ((MAX_FALL_RET - 
				      (eval->num_ret - eval->num_rel_ret))
				     * ((float)eval->num_rel_ret / 
					(float)eval->num_rel));
    }
    if (eval->num_rel > eval->num_ret) {
        eval->R_recall_precis = (float) eval->num_rel_ret / 
                                (float)eval->num_rel;
        eval->int_R_recall_precis = (float) eval->num_rel_ret / 
                                    (float)eval->num_rel;
        for (i = eval->num_ret; i < eval->num_rel; i++) {
            eval->av_R_precis += (float) eval->num_rel_ret / 
                                 (float) i;
            eval->int_av_R_precis += (float) eval->num_rel_ret / 
                                     (float) i;
        }
    }
    for (i = 0; i < NUM_PREC_PTS; i++) {
        if (eval->num_ret < cut_rprec[i]) {
            eval->R_prec_cut[i] = (float) eval->num_rel_ret / 
                (float) cut_rprec[i];
            eval->int_R_prec_cut[i] = (float) eval->num_rel_ret / 
                (float) cut_rprec[i];
        }
    }

    /* The following cutoffs/averages are correct, since 0.0 should
       be averaged in for the non-retrieved docs:
       av_recall_precis, int_av_recall_prec, int_recall_prec, 
       int_av3_recall_precis, int_av11_recall_precis
    */


    /* Calculate other indirect evaluation measure averages. */
    /* average recall-precis of 3 and 11 intermediate points */
    eval->int_av3_recall_precis =
        (eval->int_recall_precis[three_pts[0]] +
         eval->int_recall_precis[three_pts[1]] +
         eval->int_recall_precis[three_pts[2]]) / 3.0;
    for (i = 0; i < NUM_RP_PTS; i++) {
        eval->int_av11_recall_precis += eval->int_recall_precis[i];
    }
    eval->int_av11_recall_precis /= NUM_RP_PTS;

    /* Calculate judgement based measures (dependent on only
       judged docs; no assumption of non-relevance if not judged) */
    /* Binary Preference measures; here expressed as all docs with a higher 
       value of rel are to be preferred.  Optimize by keeping track of nonrel
       seen so far */
    pref_top_nonrel_num = PREF_TOP_NONREL_NUM;
    pref_top_50pRnonrel_num = 50 + eval->num_rel;
    pref_top_25pRnonrel_num = 25 + eval->num_rel;
    pref_top_10pRnonrel_num = 10 + eval->num_rel;
    pref_top_Rnonrel_num = eval->num_rel;
    pref_top_25p2Rnonrel_num = 25 + (2 * eval->num_rel);
    nonrel_ret = 0;
    for (j = 0; j < tr_vec->num_tr; j++) {
	if (tr_vec->tr[j].rel == 0)
	    nonrel_ret++;
    }
    nonrel_so_far = 0;
    rel_so_far = 0;
    bounded_5R_nonrel_so_far = 0; 
    bounded_10R_nonrel_so_far = 0; 
    for (j = 0; j < tr_vec->num_tr; j++) {
	if (tr_vec->tr[j].rel == 0) {
	    if (nonrel_so_far < 5 * eval->num_rel) {
		bounded_5R_nonrel_so_far++;
		if (nonrel_so_far < 10 * eval->num_rel) {
		    bounded_10R_nonrel_so_far++;
		}
	    }
	    nonrel_so_far++;
	}
	else if (tr_vec->tr[j].rel >= epi->relevance_level) {
	    rel_so_far++;
	    /* Add fraction of correct preferences. */
	    /* Special case nonrel_so_far == 0 to avoid division by 0 */
	    if (nonrel_so_far > 0) {
		eval->bpref_allnonrel += 1.0 - (((float) nonrel_so_far) /
					       (float) num_nonrel);
		eval->bpref_retnonrel += 1.0 - (((float) nonrel_so_far) /
					       (float) nonrel_ret);
		eval->bpref_retall += 1.0 - (((float) nonrel_so_far) /
					    (float) nonrel_ret);
		eval->bpref_topnonrel += 1.0 - 
		    (((float) MIN (nonrel_so_far, pref_top_nonrel_num)) /
		     (float) MIN (nonrel_ret, pref_top_nonrel_num));
		eval->bpref_top50pRnonrel += 1.0 - 
		    (((float) MIN (nonrel_so_far, pref_top_50pRnonrel_num)) /
		     (float) MIN (nonrel_ret, pref_top_50pRnonrel_num));
		eval->bpref_top25pRnonrel += 1.0 - 
		    (((float) MIN (nonrel_so_far, pref_top_25pRnonrel_num)) /
		     (float) MIN (nonrel_ret, pref_top_25pRnonrel_num));
		eval->bpref_top10pRnonrel += 1.0 - 
		    (((float) MIN (nonrel_so_far, pref_top_10pRnonrel_num)) /
		     (float) MIN (nonrel_ret, pref_top_10pRnonrel_num));
		eval->bpref_topRnonrel += 1.0 - 
		    (((float) MIN (nonrel_so_far, pref_top_Rnonrel_num)) /
		     (float) MIN (nonrel_ret, pref_top_Rnonrel_num));
		eval->bpref_top25p2Rnonrel += 1.0 - 
		    (((float) MIN (nonrel_so_far, pref_top_25p2Rnonrel_num)) /
		     (float) MIN (nonrel_ret, pref_top_25p2Rnonrel_num));
		if (rel_so_far <= 5 && nonrel_so_far < 5)
		    eval->bpref_5 += 1.0 - (float) nonrel_so_far /
			(float) MIN (nonrel_ret, 5);
		if (rel_so_far <= 10 && nonrel_so_far < 10)
		    eval->bpref_10 += 1.0 - (float) nonrel_so_far /
			(float) MIN (nonrel_ret, 10);
	    }
	    else {
		eval->bpref_allnonrel += 1.0;
		eval->bpref_retnonrel += 1.0;
		eval->bpref_retall += 1.0;
		eval->bpref_topnonrel += 1.0;
		eval->bpref_top50pRnonrel += 1.0;
		eval->bpref_top25pRnonrel += 1.0;
		eval->bpref_top10pRnonrel += 1.0;
		eval->bpref_topRnonrel += 1.0;
		eval->bpref_top25p2Rnonrel += 1.0;
		if (rel_so_far <= 5)
		    eval->bpref_5 += 1.0;
		if (rel_so_far <= 10)
		    eval->bpref_10 += 1.0;

	    }
	    if (eval->num_rel > 0) {
		eval->bpref_top5Rnonrel += 1.0 -
		    (((float) bounded_5R_nonrel_so_far) /
		     (float) MIN (num_nonrel, eval->num_rel * 5));
		eval->bpref_top10Rnonrel += 1.0 -
		    (((float) bounded_10R_nonrel_so_far) /
		     (float) MIN (num_nonrel, eval->num_rel * 10));
	    }
	    eval->bpref_num_all += num_nonrel - nonrel_so_far;
	    eval->bpref_num_ret += nonrel_ret - nonrel_so_far;
	}
    }
    if (eval->num_rel) {
	eval->bpref_allnonrel /= eval->num_rel;
	eval->bpref_retnonrel /= eval->num_rel;
	eval->bpref_topnonrel /= eval->num_rel;
	eval->bpref_top5Rnonrel /= eval->num_rel;
	eval->bpref_top10Rnonrel /= eval->num_rel;
	eval->bpref_top50pRnonrel /= eval->num_rel;
	eval->bpref_top25pRnonrel /= eval->num_rel;
	eval->bpref_top10pRnonrel /= eval->num_rel;
	eval->bpref_topRnonrel /= eval->num_rel;
	eval->bpref_top25p2Rnonrel /= eval->num_rel;
	if (eval->num_rel_ret)
	    eval->bpref_retall /= eval->num_rel_ret;
	eval->bpref_5 /= MIN (rel_so_far, 5);
	eval->bpref_10 /= MIN (rel_so_far, 10);
    }


    /* Calculate all the other averages */
    if (eval->num_rel_ret > 0) {
        eval->av_recall_precis /= eval->num_rel;
        eval->int_av_recall_precis /= eval->num_rel;
    }

    eval->av_fall_recall /= MAX_FALL_RET;

    eval->av_rel_precis /= eval->num_ret;
    eval->av_rel_uap /= eval->num_ret;

    if (eval->num_rel) {
        eval->av_R_precis /= eval->num_rel;
        eval->int_av_R_precis /= eval->num_rel;
        eval->exact_recall = (double) eval->num_rel_ret / eval->num_rel;
        eval->exact_precis = (double) eval->num_rel_ret / eval->num_ret;
	eval->exact_uap = eval->exact_recall * eval->exact_precis;
        if (eval->num_rel > eval->num_ret) {
            eval->exact_rel_precis = eval->exact_precis;
	}
        else {
            eval->exact_rel_precis = eval->exact_recall;
	}
	eval->exact_rel_uap = eval->exact_precis * eval->exact_precis;
	eval->exact_utility =
		epi->utility_a * eval->num_rel_ret +
		epi->utility_b * (eval->num_ret - eval->num_rel_ret) +
		epi->utility_c * (eval->num_rel - eval->num_rel_ret) +
		epi->utility_d * (epi->num_docs_in_coll + eval->num_rel_ret
				 - eval->num_ret - eval->num_rel);
    }

    if (epi->time_flag) {
	eval->time_cum_rel[0] = eval->time_num_rel[0];
	eval->av_time_cum_rel = eval->time_num_rel[0];
	for (i=1; i< NUM_TIME_PTS; i++) {
	    eval->time_cum_rel[i] = eval->time_cum_rel[i-1] + eval->time_num_rel[i];
	    eval->av_time_cum_rel += eval->time_cum_rel[i];
	    eval->av_time_precis += eval->time_precis[i];
	    eval->av_time_relprecis += eval->time_relprecis[i];
	    eval->av_time_uap += eval->time_uap[i];
	    eval->av_time_reluap += eval->time_reluap[i];
	    eval->av_time_utility += eval->time_utility[i];
	}
	eval->av_time_cum_rel /= NUM_TIME_PTS;
	eval->av_time_precis /= NUM_TIME_PTS;
	eval->av_time_relprecis /= NUM_TIME_PTS;
	eval->av_time_uap /= NUM_TIME_PTS;
	eval->av_time_reluap /= NUM_TIME_PTS;
	eval->av_time_utility /= NUM_TIME_PTS;
    }

    return (1);
}

static int
compare_iter_rank (tr1, tr2)
TR_TUP *tr1;
TR_TUP *tr2;
{
    if (tr1->iter > tr2->iter)
        return (-1);
    if (tr1->iter < tr2->iter)
        return (1);
    if (tr1->rank < tr2->rank)
        return (-1);
    if (tr1->rank > tr2->rank)
        return (1);
    return (0);
}
