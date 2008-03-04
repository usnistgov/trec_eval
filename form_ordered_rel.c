#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "trec_format.h"

/* Takes the top docs and judged docs for a query, and returns a
   rel_rank object giving the ordered relevance values for retrieved
   docs, plus relevance occurrence statatistics.
   Relevance value is
       value in text_qrels if docno is in text_qrels and was judged
           (assumed to be a small non-negative integer)
       RELVALUE_NONPOOL (-1) if docno is not in text_qrels
       RELVALUE_UNJUDGED (-2) if docno is in text_qrels and was not judged.

   This procedure may be called repeatedly for a given topic - returned
   values are cached until the query changes.

   results and rel_info formats must be "trec_results" and "qrels"
   respectively.  

   UNDEF returned if error, 0 if used cache values, 1 if new values.
*/

static int comp_rank_judged (), comp_sim_docno (), comp_docno ();

/* Definitions used for temporary and cached values */
typedef struct {
    char *docno;
    float sim;
    long rank;
    long rel;
} DOCNO_INFO;

typedef struct {
    /* Statistics about relevance occurrences in ranked_rel_list. */
    long num_nonpool;      /* Number of docs in ranked_rel list not in pool */
    long num_unjudged;     /* Number of docs in ranked_rel list not judged */
    long num_binrel_ret;   /* Number of ranked docs with rel >= 
			      epi.rel_threshold */

    /* Statistics about relevance occurrences in rel_info. */
    long num_rel_levels;   /* Number of judged rel_levels */
    long *rel_levels;      /* Number of docs in each judged rel_level 
			      (0 through num_rel_levels-1), whether
			      ranked or not */
    long num_rel;          /* Number of rel docs in rel_info */
    
    long num_ranked_rel_list; /* Number of ranked rel list */
} REL_COUNT;

/* Current cached query */
static char current_query[MAX_LEN_QUERY] = "no query";

/* Space reserved for cached returned values */
static REL_COUNT rel_count;
static long max_num_ranked_list = 0;
static long *ranked_rel_list;

int
form_ordered_rel (const EPI *epi, const REL_INFO *rel_info,
		  const RESULTS *results, RANK_REL *rank_rel)
{
    long i;
    long num_results;

    TEXT_RESULTS_INFO *text_results_info;
    TEXT_QRELS_INFO *trec_qrels;

    TEXT_QRELS *qrels_ptr, *end_qrels;

    DOCNO_INFO *docno_info;

    if (0 == strncmp (current_query, results->qid, MAX_LEN_QUERY)) {
	/* Have done this query already. Return cached values */
	rank_rel->num_rel_ret = rel_count.num_binrel_ret;
	rank_rel->num_ret = rel_count.num_ranked_rel_list;
	rank_rel->num_nonpool = rel_count.num_nonpool;
	rank_rel->num_unjudged_in_pool = rel_count.num_unjudged;
	rank_rel->num_rel = rel_count.num_rel;
	rank_rel->num_rel_levels = rel_count.num_rel_levels;
	rank_rel->rel_levels = rel_count.rel_levels;
	rank_rel->results_rel_list = ranked_rel_list;
	return (0);
    }

    /* Check that format type of result info and rel info are correct */
    if (strcmp ("qrels", rel_info->rel_format) ||
	strcmp ("trec_results", results->ret_format)) {
	fprintf (stderr, "trec_eval.form_ordered_rel: rel_info format not qrels or results format not trec_results\n");
	return (UNDEF);
    }

    (void) strncpy (current_query, results->qid, MAX_LEN_QUERY);

    text_results_info = (TEXT_RESULTS_INFO *) results->q_results;
    trec_qrels = (TEXT_QRELS_INFO *) rel_info->q_rel_info;

    num_results = text_results_info->num_text_results;

    /* Reserve space for output and temp values, if needed */
    if (num_results > max_num_ranked_list) {
        if (max_num_ranked_list > 0) 
            (void) free ((char *) ranked_rel_list);
        max_num_ranked_list += num_results;
        if (NULL == (ranked_rel_list = Malloc (max_num_ranked_list, long)))
            return (UNDEF);
    }
    if (NULL == (docno_info = Malloc (num_results, DOCNO_INFO)))
	return (UNDEF);

    for (i = 0; i < num_results; i++) {
	docno_info[i].docno = text_results_info->text_results[i].docno;
	docno_info[i].sim = text_results_info->text_results[i].sim;
    }

    /* Sort results by sim, breaking ties lexicographically using docno */
    qsort ((char *) docno_info,
	   (int) num_results,
	   sizeof (DOCNO_INFO),
	   comp_sim_docno);

    /* Only look at epi->max_num_docs_per_topic (not normally an issue) */
    if (num_results > epi->max_num_docs_per_topic)
	num_results = epi->max_num_docs_per_topic;

    /* Add ranks to docno_info (starting at 1) */
    for (i = 0; i < num_results; i++) {
        docno_info[i].rank = i+1;
    }

    /* Sort trec_top lexicographically */
    qsort ((char *) docno_info,
           (int) num_results,
           sizeof (DOCNO_INFO),
           comp_docno);

    /* Error checking for duplicates */
    for (i = 1; i < num_results; i++) {
	if (0 == strcmp (docno_info[i].docno,
			 docno_info[i-1].docno)) {
	    fprintf (stderr, "trec_eval.form_ordered_rel: duplicate docs %s",
		     docno_info[i].docno);
	    return (UNDEF);
	}
    }

    /* Go through docno_info, trec_qrels in parallel to determine relevance
       for each doc in docno_info.
       Note that trec_qrels already sorted by docno with no duplicates */
    qrels_ptr = trec_qrels->text_qrels;
    end_qrels = &trec_qrels->text_qrels[trec_qrels->num_text_qrels];
    for (i = 0; i < num_results; i++) {
        while (qrels_ptr < end_qrels &&
               strcmp (qrels_ptr->docno, docno_info[i].docno) < 0)
            qrels_ptr++;
        if (qrels_ptr >= end_qrels ||
            strcmp (qrels_ptr->docno, docno_info[i].docno) > 0) {
            /* Doc is non-judged */
            docno_info[i].rel = RELVALUE_NONPOOL;
	}
        else {
            /* Doc is in pool, assign relevance */
	    if (qrels_ptr->rel == -1)
		/* In pool, but unjudged (eg, infAP uses a sample of pool) */
		docno_info[i].rel = RELVALUE_UNJUDGED;
	    else
		docno_info[i].rel = qrels_ptr->rel;
	    qrels_ptr++;
        }
    }

    /* Construct ranked_rel_list and associated counts */
    rel_count.num_nonpool = 0;
    rel_count.num_unjudged = 0;
    rel_count.num_binrel_ret = 0;
    if (! epi->judged_docs_only_flag) {
	for (i = 0; i < num_results; i++) {
	    ranked_rel_list[docno_info[i].rank - 1] = docno_info[i].rel;
	    if (RELVALUE_NONPOOL == docno_info[i].rel)
		rel_count.num_nonpool++;
	    else if (RELVALUE_UNJUDGED == docno_info[i].rel)
		rel_count.num_unjudged++;
	    else {
		if (docno_info[i].rel >= epi->relevance_level)
		    rel_count.num_binrel_ret++;
	    }
	}
    }
    else {
	/* If judged_docs_only_flag, then must fix up ranks to reflect unjudged
	   docs being thrown out. Note: done this way to preserve original
	   tie-breaking based on text docno */
	long rrl;
	/* Sort tuples by increasing rank among judged docs*/
	qsort ((char *) docno_info,
	       (int) num_results,
	       sizeof (DOCNO_INFO),
	       comp_rank_judged);
	rrl = 0; i = 0;
	while (i < num_results && docno_info[i].rel >= 0) {
	    if (docno_info[i].rel >= epi->relevance_level)
		rel_count.num_binrel_ret++;
	    ranked_rel_list[rrl++] = docno_info[i++].rel;
	}
	num_results = rrl;
    }

    /* Fill in rank_rel returned values */
    rel_count.num_rel = 0;
    for (i = 0; i < trec_qrels->max_num_rel_levels; i++) {
	if (trec_qrels->rel_count[i]) {
	    rel_count.num_rel_levels = i + 1;
	    if (i >= epi->relevance_level)
		rel_count.num_rel += trec_qrels->rel_count[i];
	}
    }
    rel_count.rel_levels = trec_qrels->rel_count;
    rel_count.num_ranked_rel_list = num_results;

    rank_rel->num_rel_ret = rel_count.num_binrel_ret;
    rank_rel->num_ret = rel_count.num_ranked_rel_list;
    rank_rel->num_nonpool = rel_count.num_nonpool;
    rank_rel->num_unjudged_in_pool = rel_count.num_unjudged;
    rank_rel->num_rel = rel_count.num_rel;
    rank_rel->num_rel_levels = rel_count.num_rel_levels;
    rank_rel->rel_levels = rel_count.rel_levels;
    rank_rel->results_rel_list = ranked_rel_list;

    (void) free (docno_info);
    return (1);
}

static int 
comp_rank_judged (ptr1, ptr2)
DOCNO_INFO *ptr1;
DOCNO_INFO *ptr2;
{
    if (ptr1->rel >= 0 && ptr2->rel >= 0) {
	if (ptr1->rank < ptr2->rank)
	    return (-1);
	if (ptr1->rank > ptr2->rank)
	    return (1);
	return (0);
    }
    if (ptr1->rel >= 0)
	return (-1);
    if (ptr2->rel >= 0)
	return (1);
    return(0);
}

static int 
comp_sim_docno (ptr1, ptr2)
DOCNO_INFO *ptr1;
DOCNO_INFO *ptr2;
{
    if (ptr1->sim > ptr2->sim)
        return (-1);
    if (ptr1->sim < ptr2->sim)
        return (1);
    return (strcmp (ptr2->docno, ptr1->docno));
}

static int 
comp_docno (ptr1, ptr2)
DOCNO_INFO *ptr1;
DOCNO_INFO *ptr2;
{
    return (strcmp (ptr1->docno, ptr2->docno));
}
