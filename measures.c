/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/

#include "common.h"
#include "sysfunc.h"
#include "functions.h"
#include "trec_eval.h"

/* Default parameter settings for various measures */
static long long_cutoff_array[] = {5, 10, 15, 20, 30, 100, 200, 500, 1000};
static PARAMS default_P_cutoffs = {
    NULL,
    sizeof (long_cutoff_array) / sizeof (long_cutoff_array[0]),
    &long_cutoff_array[0]};
static PARAMS default_recall_cutoffs = {
    NULL,
    sizeof (long_cutoff_array) / sizeof (long_cutoff_array[0]),
    &long_cutoff_array[0]};
static PARAMS default_ndcg_cutoffs = {
    NULL,
    sizeof (long_cutoff_array) / sizeof (long_cutoff_array[0]),
    &long_cutoff_array[0]};
static PARAMS default_relative_P_cutoffs = {
    NULL,
    sizeof (long_cutoff_array) / sizeof (long_cutoff_array[0]),
    &long_cutoff_array[0]};
static PARAMS default_P_avgjg_cutoffs = {
    NULL,
    sizeof (long_cutoff_array) / sizeof (long_cutoff_array[0]),
    &long_cutoff_array[0]};
static double float_cutoff_array[] = {
    0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0};
static PARAMS default_iprec_at_recall_cutoffs = {
    NULL,
    sizeof (float_cutoff_array) / sizeof (float_cutoff_array[0]),
    &float_cutoff_array[0]};
static PARAMS default_11ptavg_cutoffs = {
    NULL,
    sizeof (float_cutoff_array) / sizeof (float_cutoff_array[0]),
    &float_cutoff_array[0]};
static double Rprec_cutoff_array[] = {
    0.2, 0.4,  0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0};
static PARAMS default_Rprec_cutoffs = {
    NULL,
    sizeof (Rprec_cutoff_array) / sizeof (Rprec_cutoff_array[0]),
    &Rprec_cutoff_array[0]};
static PARAMS default_Rprec_avgjg_cutoffs = {
    NULL,
    sizeof (Rprec_cutoff_array) / sizeof (Rprec_cutoff_array[0]),
    &Rprec_cutoff_array[0]};
static double utility_param_array[] = {1.0, -1.0, 0.0, 0.0};
static PARAMS default_utility_params = {
    NULL,
    sizeof (utility_param_array) / sizeof (utility_param_array[0]),
    &utility_param_array[0]};
static double set_F_param_array[] = {1.0};
static PARAMS default_set_F_params = {
    NULL,
    sizeof (set_F_param_array) / sizeof (set_F_param_array[0]),
    &set_F_param_array[0]};
static long success_cutoff_array[] = {1, 5, 10};
static PARAMS default_success_cutoffs = {
    NULL,
    sizeof (success_cutoff_array) / sizeof (success_cutoff_array[0]),
    &success_cutoff_array[0]};
static PARAMS default_ndcg_gains = { NULL, 0, NULL};


/* Actual measures.  Definition of TREC_MEAS below is from "trec_eval.h".
   Function prototypes for each function are defined in "functions.h".
   Code implementing each measure is normally found in "m_<meas_name>.c".
   Code implementing standard init, acc, print routines are found in
   "meas_{init,acc,print}.c".
   Measures are calculated and printed out in the order they occur here;
   the first measures tend to be the more important. */

/* Measure definition:
    typedef struct trec_meas {
        * Nmae of measure (or root name of set of measures) *
        char *name;
        * Full explanation of measure, printed upon help request *
        char *explanation;
        * Store parameters for measure in meas_params. Reserve space in
           TREC_EVAL.values for results of measure. Store individual measure
           names (possibly altered by parameters) in TREC_EVAL.values and
           initialize value to 0.0.
           Set tm->eval_index to start of reserved space *
        int (* init_meas) (EPI *epi, struct trec_meas *tm, TREC_EVAL *eval);
        * Calculate actual measure for single query *
        int (* calc_meas) (const EPI *epi, const REL_INFO *rel,
    		       const RESULTS *results,  const struct trec_meas *tm,
    		       TREC_EVAL *eval);
        * Merge info for single query into summary info *
        int (* acc_meas) (const EPI *epi, const struct trec_meas *tm,
    		      const TREC_EVAL *q_eval, TREC_EVAL *summ_eval);
        * Calculate final averages (if needed)  from summary info *
        int (* calc_avg) (const EPI *epi, const struct trec_meas *tm,
    		      const TREC_EVAL *eval);
    
        * Measure dependent parameters, defaults given here can normally be
           overridden from command line by init_meas procedure *
        void *meas_params;    
         * Index within TREC_EVAL.values for values for measure.
    	-1 indicates measure not to be calculated (default).
    	-2 indicates measure to be calculated, but has not yet been initialized.
            Set in init_meas *
        long eval_index;
    } TREC_MEAS;
*/

TREC_MEAS te_trec_measures[] = {
    {"num_q",
     "    Number of topics results averaged over.  May be different from\n\
    number of topics in the results file if -c was used on the command line \n\
    in which case number of topics in the rel_info file is used.\n",
     te_init_meas_s_long_summ,
     te_calc_num_q,
     te_acc_meas_s,
     te_calc_avg_num_q,
     NULL, -1},
    {"num_ret",
     "    Number of documents retrieved for topic. \n\
    May be affected by Judged_docs_only and Max_retrieved_per_topic command\n\
    line parameters (as are most measures).\n\
    Summary figure is sum of individual topics, not average.\n",
     te_init_meas_s_long,
     te_calc_num_ret,
     te_acc_meas_s,
     te_calc_avg_meas_empty,
     NULL, -1},
    {"num_rel",
     "    Number of relevant documents for topic. \n\
    May be affected by Judged_docs_only and Max_retrieved_per_topic command\n\
    line parameters (as are most measures).\n\
    Summary figure is sum of individual topics, not average.\n",
     te_init_meas_s_long,
     te_calc_num_rel,
     te_acc_meas_s,
     te_calc_avg_meas_empty,
     NULL, -1},
    {"num_rel_ret",
     "    Number of relevant documents retrieved for topic. \n\
    May be affected by Judged_docs_only and Max_retrieved_per_topic command\n\
    line parameters (as are most measures).\n\
    Summary figure is sum of individual topics, not average.\n",
     te_init_meas_s_long,
     te_calc_num_rel_ret,
     te_acc_meas_s,
     te_calc_avg_meas_empty,
     NULL, -1},
    {"map",
     "    Mean Average Precision\n\
    Precision measured after each relevant doc is retrieved, then averaged\n\
    for the topic, and then averaged over topics (if more than one).\n\
    This is the main single-valued number used to compare the entire rankings\n\
    of two or more retrieval methods.  It has proven in practice to be useful\n\
    and robust.\n\
    The name of the measure is unfortunately inaccurate since it is \n\
    calculated for a single topic (and thus don't want both 'mean' and\n\
    'average') but was dictated by common usage and the need to distiguish\n\
    map from Precision averaged over topics (I had to give up my attempts to\n\
    call it something else!)\n\
    History: Developed by Chris Buckley after TREC 1.\n\
    Cite: 'Retrieval System Evaluation', Chris Buckley and Ellen Voorhees.\n\
    Chapter 3 in TREC: Experiment and Evaluation in Information Retrieval\n\
    edited by Ellen Voorhees and Donna Harman.  MIT Press 2005\n",
     te_init_meas_s_float,
     te_calc_map,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"gm_map",
     "    Geometric Mean Average Precision\n\
    This is the same measure as 'map' (see description of 'map') on an\n\
    individual topic, but the geometric mean is calculated when averaging\n\
    over topics.  This rewards methods that are more consistent over topics\n\
    as opposed to methods which do very well for some topics but very poorly\n\
    for others.\n\
    gm_ap is reported only in the summary over all topics, not for individual\n\
    topics.\n",
     te_init_meas_s_float_summ,
     te_calc_gm_map,
     te_acc_meas_s,
     te_calc_avg_meas_s_gm,
     NULL, -1},
    {"Rprec",
    "    Precision after R documents have been retrieved.\n\
    R is the total number of relevant docs for the topic.  \n\
    This is a good single point measure for an entire retrieval\n\
    ranking that averages well since each topic is being averaged\n\
    at an equivalent point in its result ranking.\n\
    Note that this is the point that Precision = Recall.\n\
    History: Originally developed for IR rankings by Chris Buckley\n\
    after TREC 1, but analogs were used in other disciplines previously.\n\
    (the point where P = R is an important one!)\n\
    Cite: 'Retrieval System Evaluation', Chris Buckley and Ellen Voorhees.\n\
    Chapter 3 in TREC: Experiment and Evaluation in Information Retrieval\n\
    edited by Ellen Voorhees and Donna Harman.  MIT Press 2005\n",
     te_init_meas_s_float,
     te_calc_Rprec,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"bpref",
     "    Main binary preference measure.\n\
    Fraction of the top R nonrelevant docs that are retrieved after each\n\
    relevant doc. Put another way: when looking at the R relevant docs, and\n\
    the top R nonrelevant docs, if all relevant docs are to be preferred to\n\
    nonrelevant docs, bpref is the fraction of the preferences that the\n\
    ranking preserves.\n\
    Cite: 'Retrieval Evaluation with Incomplete Information', Chris Buckley\n\
    and Ellen Voorhees. In Proceedings of 27th SIGIR, 2004.\n",
     te_init_meas_s_float,
     te_calc_bpref,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"recip_rank",
     "    Reciprocal Rank of the first relevant retrieved doc.\n\
    Measure is most useful for tasks in which there is only one relevant\n\
    doc, or the user only wants one relevant doc.\n",
     te_init_meas_s_float,
     te_calc_recip_rank,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"iprec_at_recall",
     "    Interpolated Precision at recall cutoffs.\n\
    This is the data shown in the standard Recall-Precision graph.\n\
    The standard cutoffs and interpolation are needed to average data over\n\
    multiple topics; otherwise, how is a topic with 5 relevant docs averaged\n\
    with a topic with 3 relevant docs for graphing purposes?  The Precision \n\
    interpolation used here is\n\
      Int_Prec (rankX) == MAX (Prec (rankY)) for all Y >= X.\n\
    Default usage: -m iprec_at_recall.0,.1,.2,.3,.4,.5,.6,.7,.8,.9,1 ...\n",
     te_init_meas_a_float_cut_float,
     te_calc_iprec_at_recall,
     te_acc_meas_a_cut, 
     te_calc_avg_meas_a_cut,
     (void *) &default_iprec_at_recall_cutoffs, -1},
    {"P",
     "    Precision at cutoffs\n\
    Precision measured at various doc level cutoffs in the ranking.\n\
    If the cutoff is larger than the number of docs retrieved, then\n\
    it is assumed nonrelevant docs fill in the rest.  Eg, if a method\n\
    retrieves 15 docs of which 4 are relevant, then P20 is 0.2 (4/20).\n\
    Precision is a very nice user oriented measure, and a good comparison\n\
    number for a single topic, but it does not average well. For example,\n\
    P20 has very different expected characteristics if there 300\n\
    total relevant docs for a topic as opposed to 10.\n\
    Note:   trec_eval -m P.50 ...\n\
    is different from \n\
            trec_eval -M 50 -m set_P ...\n\
    in that the latter will not fill in with nonrel docs if less than 50\n\
    docs retrieved\n\
    Cutoffs must be positive without duplicates\n\
    Default param: -m P.5,10,15,20,30,100,200,500,1000\n",
     te_init_meas_a_float_cut_long,
     te_calc_P,
     te_acc_meas_a_cut,
     te_calc_avg_meas_a_cut,
     (void *) &default_P_cutoffs, -1},
    {"recall",
     "    Recall at cutoffs\n\
    Recall (relevant retrieved / relevant) measured at various doc level\n\
    cutoffs in the ranking. If the cutoff is larger than the number of docs\n\
    retrieved, then it is assumed nonrelevant docs fill in the rest.\n\
    REcall is a fine single topic measure, but does not average well.\n\
    Cutoffs must be positive without duplicates\n\
    Default param: -m recall.5,10,15,20,30,100,200,500,1000\n",
     te_init_meas_a_float_cut_long,
     te_calc_recall,
     te_acc_meas_a_cut,
     te_calc_avg_meas_a_cut,
     (void *) &default_recall_cutoffs, -1},
    {"infAP",
     "    Inferred AP\n\
    A measure that allows sampling of judgement pool: Qrels/results divided\n\
    into unpooled, pooled_but_unjudged, pooled_judged_rel,pooled_judged_nonrel.\n\
    My intuition of infAP:\n\
    Assume a judgment pool with a random subset that has been judged.\n\
    Calculate P at rel doc using only the judged higher retrieved docs,\n\
    then average in 0's from higher docs that were not in the judgment pool.\n\
    (Those in the pool but not judged are ignored, since they are assumed\n\
    to be relevant in the same proportion as those judged.)\n\
    Cite:    'Estimating Average Precision with Incomplete and Imperfect\n\
    Judgments', Emine Yilmaz and Javed A. Aslam. CIKM \n",
     te_init_meas_s_float,
     te_calc_infap,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"gm_bpref",
     "   Binary preference (bpref), but using goemetric mean over topics\n\
    See the explanation for 'bpref' for the base measure for a single topic.\n\
    Gm_bpref uses the geometric mean to combine the single topic scores.\n\
    This rewards methods that are more consistent across topics as opposed to\n\
    high scores for some topics and low scores for others.\n\
    Gm_bpref is printed only as a summary measure across topics, not for the\n\
    individual topics.\n",
     te_init_meas_s_float_summ,
     te_calc_gm_bpref,
     te_acc_meas_s,
     te_calc_avg_meas_s_gm,
     NULL, -1},
    {"Rprec_mult",
     "    Precision measured at multiples of R (num_rel).\n\
    This is an attempt to measure topics at the same multiple milestones\n\
    in a retrieval (see explanation of R-prec), in order to determine\n\
    whether methods are precision oriented or recall oriented.  If method A\n\
    dominates method B at the low multiples but performs less well at the\n\
    high multiples then it is precision oriented (compared to B).\n\
    Default param: -m Rprec_mult.0.2,0.4,0.6,0.8,1.0,1.2,1.4,1.6,1.8,2.0 ...\n",
     te_init_meas_a_float_cut_float,
     te_calc_Rprec_mult,
     te_acc_meas_a_cut,
     te_calc_avg_meas_a_cut,
     (void *) &default_Rprec_cutoffs, -1},
    {"utility",
     "    Set utility measure\n\
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
    Default usage: -m utility.1.0,-1.0,0.0,0.0 ...\n\
    Warning: Current version summary evaluation averages over all topics;\n\
    it could be argued that simply summing is more useful (but not backward\n\
    compatible)\n",
     te_init_meas_s_float_p_float,
     te_calc_utility,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     (void *) &default_utility_params, -1},
    {"11pt_avg",
     "    Interpolated Precision averaged over 11 recall points\n\
    Obsolete, only use for comparisons of old runs; should use map instead.\n\
    Average interpolated at the given recall points - default is the\n\
    11 points being reported for ircl_prn.\n\
    Both map and 11-pt_avg (and even R-prec) can be regarded as estimates of\n\
    the area under the standard ircl_prn curve.\n\
    Warning: name assumes user does not change default parameter values:\n\
    measure name is independent of parameter values and number of parameters.\n\
    Will actually average over all parameter values given.\n\
    To get 3-pt_avg as in trec_eval version 8 and earlier, use\n\
      trec_eval -m 11-pt_avg.0.2,0.5,0.8 ...\n\
    Default usage: -m 11-pt_avg.0.0,.1,.2,.3,.4,.5,.6,.7,.8..9,1.0\n",
     te_init_meas_s_float_p_float,
     te_calc_11ptavg,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     &default_11ptavg_cutoffs, -1},
    {"ndcg",
     "    Normalized Discounted Cumulative Gain\n\
    Compute a traditional nDCG measure according to Jarvelin and\n\
    Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002).\n\
    Gain values are the relevance values in the qrels file.  For now, if you\n\
    want different gains, change the qrels file appropriately, or use\n\
    ndcg_p.  Will be replaced by ndcg_p? ... \n",
     te_init_meas_s_float,
     te_calc_ndcg,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"ndcg_p",
     "    Normalized Discounted Cumulative Gain\n\
    Compute a traditional nDCG measure according to Jarvelin and\n\
    Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002).\n\
    Gain values are set to the appropriate relevance level by default.  \n\
    The default gain can be overridden on the command line by having \n\
    comma separated parameters 'rel_level=gain'.\n\
    Eg, 'trec_eval -m ndcg_p.1=3.5,2=9.0,4=7.0 ...'\n\
    will give gains 3.5, 9.0, 3.0, 7.0 for relevance levels 1,2,3,4\n\
    respectively (level 3 remains at the default).\n\
    Gains are allowed to be 0 or negative, and relevance level 0\n\
    can be given a gain.\n\
    Based on an implementation by Ian Soboroff\n",
     te_init_meas_s_float_p_pair,
     te_calc_ndcg_p,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     &default_ndcg_gains, -1},
    {"ndcg_cut",
     "    Normalized Discounted Cumulative Gain at cutoffs.\n\
    Compute a traditional nDCG measure according to Jarvelin and\n\
    Kekalainen (ACM ToIS v. 20, pp. 422-446, 2002) at cutoffs.\n\
    See comments for ndcg.\n\
    Gain values are the relevance values in the qrels file.  For now, if you\n\
    want different gains, change the qrels file appropriately.\n\
    Cutoffs must be positive without duplicates\n\
    Default params: -m ndcg_cut.5,10,15,20,30,100,200,500,1000\n\
    Based on an implementation by Ian Soboroff\n",
     te_init_meas_a_float_cut_long,
     te_calc_ndcg_cut,
     te_acc_meas_a_cut,
     te_calc_avg_meas_a_cut,
     (void *) &default_ndcg_cutoffs, -1},
    {"relative_P",
     "    Relative Precision at cutoffs\n\
    Precision at cutoff relative to the maximum possible precision at that\n\
    cutoff.  Equivalent to Precision up until R, and then recall after R\n\
    Cutoffs must be positive without duplicates\n\
    Default params: -m relative_P.5,10,15,20,30,100,200,500,1000\n",
     te_init_meas_a_float_cut_long,
     te_calc_rel_P,
     te_acc_meas_a_cut,
     te_calc_avg_meas_a_cut,
     (void *) &default_relative_P_cutoffs, -1},
    {"success",
     "    Success at cutoffs\n\
    Success (a relevant doc has been retrieved) measured at various doc level\n\
    cutoffs in the ranking.\n\
    If the cutoff is larger than the number of docs retrieved, then\n\
    it is assumed nonrelevant docs fill in the rest.\n\
    Cutoffs must be positive without duplicates\n\
    Default param: trec_eval -m success.1,5,10\n\
    History: Developed by Stephen Tomlinson.\n",
     te_init_meas_a_float_cut_long,
     te_calc_success,
     te_acc_meas_a_cut,
     te_calc_avg_meas_a_cut,
     (void *) &default_success_cutoffs, -1},
    {"set_P",
     "    Set Precision: num_relevant_retrieved / num_retrieved \n\
    Precision over all docs retrieved for a topic.\n\
    Was known as exact_prec in earlier versions of trec_eval\n\
    Note:   trec_eval -m P.50 ...\n\
    is different from \n\
            trec_eval -M 50 -m set_P ...\n\
    in that the latter will not fill in with nonrel docs if less than \n\
    50 docs retrieved\n",
     te_init_meas_s_float,
     te_calc_set_P,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"set_relative_P",
     "    Relative Set Precision:  P / (Max possible P for this size set) \n\
    Relative precision over all docs retrieved for a topic.\n\
    Was known as exact_relative_prec in earlier versions of trec_eval\n\
    Note:   trec_eval -m relative_P.50 ...\n\
    is different from \n\
            trec_eval -M 50 -m set_relative_P ...\n",
     te_init_meas_s_float,
     te_calc_set_relative_P,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"set_recall",
     "    Set Recall: num_relevant_retrieved / num_relevant \n\
    Recall over all docs retrieved for a topic.\n\
    Was known as exact_recall in earlier versions of trec_eval",
     te_init_meas_s_float,
     te_calc_set_recall,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"set_map",
     "    Set map: num_relevant_retrieved**2 / (num_retrieved*num_rel)\n\
    Unranked set map, where the precision due to all relevant retrieved docs\n\
    is the set precision, and the precision due to all relevant not-retrieved\n\
    docs is set to 0.\n\
    Was known as exact_unranked_avg_prec in earlier versions of trec_eval.\n\
    Another way of loooking at this is  Recall * Precision on the set of\n\
    docs retrieved for a topic.\n",
     te_init_meas_s_float,
     te_calc_set_map,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"set_F",
     "      Set F measure: weighted harmonic mean of recall and precision\n\
    set_Fx = (x+1) * P * R / (R + x*P)\n\
    where x is the relative importance of R to P (default 1.0).\n\
    Default usage: trec_eval -m set_F.1.0 ...\n\
    Cite: Variant of van Rijsbergen's E measure ('Information Retrieval',\n\
    Butterworths, 1979).\n",
     te_init_meas_s_float_p_float,
     te_calc_set_F,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     (void *) &default_set_F_params, -1},
    {"num_nonrel_judged_ret",
     "    Number of non-relevant judged documents retrieved for topic. \n\
    Not an evaluation number per se, but gives details of retrieval results.\n\
    Summary figure is sum of individual topics, not average.\n",
     te_init_meas_s_long,
     te_calc_num_nonrel_judged_ret,
     te_acc_meas_s, 
     te_calc_avg_meas_empty,
     NULL, -1},
    {"prefs_num_prefs_poss",
     "    Number of possible prefs independent of whether documents retrieved\n\
    Summary figure is sum of individual topics, not average.\n",
     te_init_meas_s_long,
     te_calc_prefs_num_prefs_poss,
     te_acc_meas_s, 
     te_calc_avg_meas_empty,
     NULL, -1},
    {"prefs_num_prefs_ful",
     "    Number of prefs fulfilled\n\
    For doc pref A>B, this includes implied preferences (only one of A or B\n\
    retrieved), and counts as failure if neither A nor B retrieved.\n\
    Summary figure is sum of individual topics, not average.\n",
     te_init_meas_s_long,
     te_calc_prefs_num_prefs_ful,
     te_acc_meas_s, 
     te_calc_avg_meas_empty,
     NULL, -1},
    {"prefs_num_prefs_ful_ret",
     "    Number of prefs fulfilled among retrieved docs\n\
    For doc pref A>B, both A nd B must be retrieved to be counted.\n\
    Summary figure is sum of individual topics, not average.\n",
     te_init_meas_s_long,
     te_calc_prefs_num_prefs_ful_ret,
     te_acc_meas_s, 
     te_calc_avg_meas_empty,
     NULL, -1},
    {"prefs_simp",
    "    Simple ratio of preferences fulfilled to preferences possible.\n\
    If a doc pair satisfies two preferences, both are counted.\n\
    If preferences are conflicted for a doc pair, all are counted\n\
    (and thus max possible score may be less than 1.0 for topic).\n\
    For doc pref A>B, this includes implied preferences (only one of A or B\n\
    retrieved), and counts as failure if neither A nor B retrieved.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_simp,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_pair",
     "   Average over doc pairs of preference ratio for that pair.\n\
    If a doc pair satisfies 3 preferences but fails 2 preferences (preferences\n\
    from 5 different users),  then the score for doc pair is 3/5.\n\
    Same as prefs_simp if there are no doc_pairs in multiple judgment groups.\n\
    For doc pref A>B, this includes implied preferences (only one of A or B\n\
    retrieved), and counts as failure if neither A nor B retrieved.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_pair,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_avgjg",
     "    Simple ratio of preferences fulfilled to preferences possible\n\
    within a judgment group, averaged over jgs.  I.e., rather than considering\n\
    all preferences equal (prefs_simp), consider all judgment groups equal.\n\
    prefs_avgjg = AVERAGE_OVER_JG (fulfilled_jg / possible_jg);\n\
    May be useful in applications where user satisfaction is represented\n\
    by a jg per user, and it is not desirable for many preferences expressed\n\
    by user1 to swamp a few preferences by user2.\n\
    For doc pref A>B, this includes implied preferences (only one of A or B\n\
    retrieved), and counts as failure if neither A nor B retrieved.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_avgjg,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_avgjg_Rnonrel",
     "    Ratio of preferences fulfilled to preferences possible within a\n\
    judgment group, averaged over jgs, except that the number of\n\
    nonrelevant retrieved docs (rel_level == 0.0) in each jg is set to\n\
    R, the number of relevant retrieved docs (rel_level > 0.0) in that jg.\n\
    \n\
    This addresses the general problem that the number of\n\
    nonrelevant docs judged for a topic can be critical to fair\n\
    evaluation - adding a couple of hundred preferences involving\n\
    nonrelevant docs (out of the possibly millions in a collection) can\n\
    both change the importance of the topic when averaging and even\n\
    change whether system A scores better than system B (even given\n\
    identical retrieval on the added nonrel docs).\n\
    \n\
    This measure conceptually sets the number of nonrelevant retrieved\n\
    docs of a jg to R. If the actual number, N, is less than R, then R\n\
    * (R-N) fulfilled preferences are added.  If N is greater than R,\n\
    then only the first R (rank order) docs in the single ec with\n\
    rel_level = 0.0 are used and the number of preferences are\n\
    recalculated.  \n\
    If there is a single jg with two equivalence classes (one of them 0.0), \n\
    then prefs_avgjg_Rnonrel is akin to the ranked measure bpref.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_avgjg_Rnonrel,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_simp_ret",
    "    Simple ratio of preferences fulfilled to preferences possible among.\n\
    the retrieved docs. \n\
    If a doc pair satisfies two preferences, both are counted.\n\
    If preferences are conflicted for a doc pair, all are counted\n\
    (and thus max possible score may be less than 1.0 for topic).\n\
    For doc pref A>B, A and B must both be retrieved to be counted as either\n\
    fulfilled or possible.\n\
    pref_*_ret measures should be used for dynamic collections but are\n\
    inferior in most other applications.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_simp_ret,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_pair_ret",
     "   Average over doc pairs of preference ratio for that pair.\n\
    If a doc pair satisfies 3 preferences but fails 2 preferences (preferences\n\
    from 5 different users),  then the score for doc pair is 3/5.\n\
    Same as prefs_simp if there are no doc_pairs in multiple judgment groups.\n\
    For doc pref A>B, A and B must both be retrieved to be counted as either\n\
    fulfilled or possible.\n\
    For doc pref A>B, this includes implied preferences (only one of A or B\n\
    retrieved), and counts as failure if neither A nor B retrieved.\n\
    pref_*_ret measures should be used for dynamic collections but are\n\
    inferior in most other applications.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_pair_ret,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_avgjg_ret",
     "    Simple ratio of preferences fulfilled to preferences possible\n\
    within a judgment group, averaged over jgs.  I.e., rather than considering\n\
    all preferences equal (prefs_simp), consider all judgment groups equal.\n\
    prefs_avgjg = AVERAGE_OVER_JG (fulfilled_jg / possible_jg);\n\
    May be useful in applications where user satisfaction is represented\n\
    by a jg per user, and it is not desirable for many preferences expressed\n\
    by user1 to swamp a few preferences by user2.\n\
    For doc pref A>B, A and B must both be retrieved to be counted as either\n\
    fulfilled or possible.\n\
    pref_*_ret measures should be used for dynamic collections but are\n\
    inferior in most other applications.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_avgjg_ret,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_avgjg_Rnonrel_ret",
     "    Ratio of preferences fulfilled to preferences possible within a\n\
    judgment group, averaged over jgs, except that the number of\n\
    nonrelevant retrieved docs (rel_level == 0.0) in each jg is set to\n\
    R, the number of relevant retrieved docs (rel_level > 0.0) in that jg.\n\
    \n\
    This addresses the general problem that the number of\n\
    nonrelevant docs judged for a topic can be critical to fair\n\
    evaluation - adding a couple of hundred preferences involving\n\
    nonrelevant docs (out of the possibly millions in a collection) can\n\
    both change the importance of the topic when averaging and even\n\
    change whether system A scores better than system B (even given\n\
    identical retrieval on the added nonrel docs).\n\
    \n\
    This measure conceptually sets the number of nonrelevant retrieved\n\
    docs of a jg to R. If the actual number, N, is less than R, then R\n\
    * (R-N) fulfilled preferences are added.  If N is greater than R,\n\
    then only the first R (rank order) docs in the single ec with\n\
    rel_level = 0.0 are used and the number of preferences are\n\
    recalculated.  \n\
    If there is a single jg with two equivalence classes (one of them 0.0), \n\
    then prefs_avgjg_Rnonrel is akin to the ranked measure bpref.\n\
    For doc pref A>B, A and B must both be retrieved to be counted as either\n\
    fulfilled or possible.\n\
    pref_*_ret measures should be used for dynamic collections but are\n\
    inferior in most other applications.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_avgjg_Rnonrel_ret,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_simp_imp",
    "    Simple ratio of preferences fulfilled to preferences possible.\n\
    If a doc pair satisfies two preferences, both are counted.\n\
    If preferences are conflicted for a doc pair, all are counted\n\
    (and thus max possible score may be less than 1.0 for topic).\n\
    For doc pref A>B, this includes implied preferences (only one of A or B\n\
    retrieved), but ignores pair if neither A nor B retrieved.\n\
    pref_*_imp measures don't have any preferred applications that I know of,\n\
    but some people like them.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_simp_imp,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_pair_imp",
     "   Average over doc pairs of preference ratio for that pair.\n\
    If a doc pair satisfies 3 preferences but fails 2 preferences (preferences\n\
    from 5 different users),  then the score for doc pair is 3/5.\n\
    Same as prefs_simp if there are no doc_pairs in multiple judgment groups.\n\
    For doc pref A>B, this includes implied preferences (only one of A or B\n\
    retrieved), but ignores pair if neither A nor B retrieved.\n\
    pref_*_imp measures don't have any preferred applications that I know of,\n\
    but some people like them.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_pair_imp,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"prefs_avgjg_imp",
     "    Simple ratio of preferences fulfilled to preferences possible\n\
    within a judgment group, averaged over jgs.  I.e., rather than considering\n\
    all preferences equal (prefs_simp), consider all judgment groups equal.\n\
    prefs_avgjg = AVERAGE_OVER_JG (fulfilled_jg / possible_jg);\n\
    May be useful in applications where user satisfaction is represented\n\
    by a jg per user, and it is not desirable for many preferences expressed\n\
    by user1 to swamp a few preferences by user2.\n\
    For doc pref A>B, this includes implied preferences (only one of A or B\n\
    retrieved), but ignores pair if neither A nor B retrieved.\n\
    pref_*_imp measures don't have any preferred applications that I know of,\n\
    but some people like them.\n\
    Assumes '-R prefs' or '-R qrels_prefs'\n",
     te_init_meas_s_float,
     te_calc_prefs_avgjg_imp,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"map_avgjg",
      "    Mean Average Precision over judgment groups \n\
    Precision measured after each relevant doc is retrieved, then averaged\n\
    for the topic, and then averaged over judgement group (user) and then \n\
    averaged over topics (if more than one).\n\
    Same as the workhorse measure 'map' except if there is more than one\n\
    set of relevance judgments for this query (each set indicated by a\n\
    different judgment group), the score will be averaged over the judgment\n\
    groups.\n",
     te_init_meas_s_float,
     te_calc_map_avgjg,
     te_acc_meas_s,
     te_calc_avg_meas_s,
     NULL, -1},
    {"P_avgjg",
     "    Precision at cutoffs, averaged over judgment groups (users)\n\
    Precision measured at various doc level cutoffs in the ranking.\n\
    If the cutoff is larger than the number of docs retrieved, then\n\
    it is assumed nonrelevant docs fill in the rest.  Eg, if a method\n\
    retrieves 15 docs of which 4 are relevant, then P20 is 0.2 (4/20).\n\
    If there are multiple relevance judgment sets for this query, Precision\n\
    is averaged over the judgment groups.\n\
    Cutoffs must be positive without duplicates\n\
    Default param: trec_eval -m P.5,10,15,20,30,100,200,500,1000\n",
     te_init_meas_a_float_cut_long,
     te_calc_P_avgjg,
     te_acc_meas_a_cut,
     te_calc_avg_meas_a_cut,
     (void *) &default_P_avgjg_cutoffs, -1},
    {"Rprec_mult_avgjg",
    "    Precision measured at multiples of R(num_rel) averged over users.\n\
    This is an attempt to measure topics at the same multiple milestones\n\
    in a retrieval (see explanation of R-prec), in order to determine\n\
    whether methods are precision oriented or recall oriented.  If method A\n\
    dominates method B at the low multiples but performs less well at the\n\
    high multiples then it is precision oriented (compared to B).\n\
    If there is more than one judgment group (set of evalutation judgments\n\
    of a user), then the measure is averaged over those jgs.\n\
    Default param: \n\
    trec_eval -m Rprec_mult_avgjg.0.2,0.4,0.6,0.8,1.0,1.2,1.4,1.6,1.8,2.0  ...\n",
     te_init_meas_a_float_cut_float,
     te_calc_Rprec_mult_avgjg,
     te_acc_meas_a_cut,
     te_calc_avg_meas_a_cut,
     (void *) &default_Rprec_avgjg_cutoffs, -1},
    
};
int te_num_trec_measures = sizeof (te_trec_measures) / sizeof (te_trec_measures[0]);

static char *off_names[] =  {
    "num_q", "num_ret", "num_rel", "num_rel_ret", "map", "gm_map", 
    "Rprec", "bpref", "recip_rank", "iprec_at_recall", "P", NULL};
static char *trec_names[] =  {
    "num_q", "num_ret", "num_rel", "num_rel_ret", "map", "gm_map", "Rprec",
    "bpref", "recip_rank", "iprec_at_recall", "P", "recall", "infAP","gm_bpref",
    "utility", "11pt_avg", "ndcg", "relative_P", "Rprec_mult", "success",
    "ndcg_cut", "ndcg_p",
    "set_P", "set_recall", "set_relative_P", "set_map", "set_F",
    "num_nonrel_judged_ret",
    NULL};
static char *set_names[] =  {
    "num_q", "num_ret", "num_rel", "num_rel_ret", "utility", "set_P",
    "set_recall", "set_relative_P", "set_map", "set_F",
    NULL};
static char *prefs_names[] =  {
    "num_q","prefs_num_prefs_poss", "prefs_num_prefs_ful",
    "prefs_num_prefs_ful_ret",
    "prefs_simp", "prefs_pair", "prefs_avgjg", "prefs_avgjg_Rnonrel",
    "prefs_simp_ret", "prefs_pair_ret", "prefs_avgjg_ret",
    "prefs_avgjg_Rnonrel_ret",
    "prefs_simp_imp", "prefs_pair_imp", "prefs_avgjg_imp",
    NULL};
static char *prefs_off_name[] =  {
    "num_q",
    "prefs_num_prefs_poss", "prefs_num_prefs_ful", "prefs_num_prefs_ful_ret",
    "prefs_simp", "prefs_pair", "prefs_avgjg",
    NULL};
static char *qrels_jg_names[] = {
    "num_q",
    "map_avgjg", "P_avgjg", "Rprec_mult_avgjg",
    NULL};

TREC_MEASURE_NICKNAMES te_trec_measure_nicknames[] = {
    {"official", off_names},
    {"set", set_names},
    {"all_trec", trec_names},
    {"all_prefs", prefs_names},
    {"prefs", prefs_off_name},
    {"qrels_jg", qrels_jg_names},
};
int te_num_trec_measure_nicknames =
    sizeof (te_trec_measure_nicknames) / sizeof (te_trec_measure_nicknames[0]);
