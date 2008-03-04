/* Copyright (c) 2003, 1991, 1990, 1984 Chris Buckley.  */


#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "trec_format.h"
#include <ctype.h>


/* Read all relevance information from text_qrels_file.
Relevance for each docno to qid is determined from text_qrels_file, which
consists of text tuples of the form
   qid  iter  docno  rel
giving TREC document numbers (docno, a string) and their relevance (rel, 
an integer) to query qid (a string).  iter string field is ignored.  
Fields are separated by whitespace, string fields can contain no whitespace.
File may contain no NULL characters.

All tuples are stored in per query arrays within all_rel_info.
Each list of query judgments is sorted lexicographically by docno,
and checked for duplicates.
Counts of rel_info are returned within all_rel_info.
*/

static int parse_qrels_line (char **start_ptr, char **qid_ptr,
			     char **docno_ptr, char **rel_ptr);

static int comp_qrels_docno ();

int 
te_get_qrels (EPI *epi, char *text_qrels_file, ALL_REL_INFO *all_rel_info)
{
    int fd;
    int size = 0;
    char *trec_qrels_buf;
    char *ptr;
    char *current_qid;
    char *qid_ptr, *docno_ptr, *rel_ptr;
    long i,j;
    long rel;
    TEXT_QRELS_INFO *current_qrels = NULL;
    long line_num;

    /* Read entire file into memory */
    if (-1 == (fd = open (text_qrels_file, 0)) ||
        -1 == (size = lseek (fd, 0L, 2)) ||
        NULL == (trec_qrels_buf = malloc ((unsigned) size+2)) ||
        -1 == lseek (fd, 0L, 0) ||
        size != read (fd, trec_qrels_buf, size) ||
	-1 == close (fd)) {
        fprintf (stderr, "trec_eval.get_qrels: Cannot read qrels file\n");
        return (UNDEF);
    }

    current_qid = "";

    /* Initialize all_rel_info */
    all_rel_info->num_q_rels = 0;
    all_rel_info->max_num_q_rels = INIT_NUM_QUERIES;
    if (NULL == (all_rel_info->rel_info = Malloc (INIT_NUM_QUERIES,
						     REL_INFO)))
	return (UNDEF);
    
    if (size == 0)
	return (0);

    /* Append ending newline if not present, Append NULL terminator */
    if (trec_qrels_buf[size-1] != '\n') {
	trec_qrels_buf[size] = '\n';
	size++;
    }
    trec_qrels_buf[size] = '\0';

    ptr = trec_qrels_buf;
    line_num = 0;
    while (*ptr) {
	line_num++;
	/* Get current line */
	if (UNDEF == parse_qrels_line (&ptr, &qid_ptr, &docno_ptr,
				       &rel_ptr)) {
	    fprintf (stderr, "trec_eval.get_qrels: Malformed line %ld\n",
		     line_num);
	    return (UNDEF);
	}
	if (0 != strcmp (qid_ptr, current_qid)) {
	    /* Query has changed. Must check if new query or this is more
	       judgements for an old query */
	    current_qid = qid_ptr;
	    for (i = 0; i < all_rel_info->num_q_rels; i++) {
		if (0 == strcmp (qid_ptr, all_rel_info->rel_info[i].qid))
		    break;
	    }
	    if (i >= all_rel_info->num_q_rels) {
		/* New unseen query, add and initialize it */
		if (all_rel_info->num_q_rels >=
		    all_rel_info->max_num_q_rels) {
		    all_rel_info->max_num_q_rels *= 10;
		    if (NULL == (all_rel_info->rel_info = 
				 Realloc (all_rel_info->rel_info,
					  all_rel_info->max_num_q_rels,
					  REL_INFO)))
			return (UNDEF);
		}
		all_rel_info->rel_info[i].qid = qid_ptr;
		all_rel_info->rel_info[i].rel_format = "qrels";
		if (NULL == (all_rel_info->rel_info[i].q_rel_info = 
			     Malloc (1, TEXT_QRELS_INFO)))
		    return (UNDEF);
		current_qrels = all_rel_info->rel_info[i].q_rel_info;
		current_qrels->num_text_qrels = 0;
		current_qrels->max_num_text_qrels = INIT_NUM_RELS;
		if (NULL == (current_qrels->text_qrels =
			     Malloc (INIT_NUM_RELS, TEXT_QRELS)))
		    return (UNDEF);
		current_qrels->max_num_rel_levels = INIT_NUM_REL_LEVELS;
		if (NULL == (current_qrels->rel_count =
			     Malloc (INIT_NUM_REL_LEVELS, long)))
		    return (UNDEF);
		memset (current_qrels->rel_count, 0,
			INIT_NUM_REL_LEVELS * sizeof (long));
		all_rel_info->num_q_rels++;
	    }
	    else {
		/* Old query, just switch current_qrels */
		current_qrels = (TEXT_QRELS_INFO *)
		    all_rel_info->rel_info[i].q_rel_info;
	    }
	}
	
	/* Add judgement to current query's list */
	if (current_qrels->num_text_qrels >= 
	    current_qrels->max_num_text_qrels) {
	    /* Need more space */
	    current_qrels->max_num_text_qrels *= 10;
	    if (NULL == (current_qrels->text_qrels = 
			 Realloc (current_qrels->text_qrels,
				  current_qrels->max_num_text_qrels,
				  TEXT_QRELS)))
		return (UNDEF);
	}
	current_qrels->text_qrels[current_qrels->num_text_qrels].docno =
		docno_ptr;
	rel = atol (rel_ptr);
	if (rel >= current_qrels->max_num_rel_levels) {
	    long temp = current_qrels->max_num_rel_levels;
	    current_qrels->max_num_rel_levels += rel;
	    if (NULL == (current_qrels->rel_count =
			 Realloc (current_qrels->rel_count,
				  current_qrels->max_num_rel_levels,
				  long)))
		return (UNDEF);
	    memset (&current_qrels->rel_count[temp], 0, rel * sizeof (long));
	}
	current_qrels->text_qrels[current_qrels->num_text_qrels++].rel =
	    rel;
	if (rel >= 0)
	    current_qrels->rel_count[rel]++;
    }

    /* Sort each query's text_qrels by docno */   
    for (i = 0; i < all_rel_info->num_q_rels; i++) {
	TEXT_QRELS_INFO *tqi = (TEXT_QRELS_INFO *)
	    all_rel_info->rel_info[i].q_rel_info;
	qsort ((char *)tqi->text_qrels,
	       (int) tqi->num_text_qrels,
	       sizeof (TEXT_QRELS),
	       comp_qrels_docno);
	for (j = 1; j < tqi->num_text_qrels; j++) {
	    if (0 == strcmp (tqi->text_qrels[j].docno,
			     tqi->text_qrels[j-1].docno)) {
		fprintf (stderr, "trec_eval.get_qrels: duplicate docs %s\n",
			 tqi->text_qrels[i].docno);
		return (UNDEF);
	    }
	}	
    }

    return (1);
}

static int 
comp_qrels_docno (ptr1, ptr2)
TEXT_QRELS *ptr1;
TEXT_QRELS *ptr2;
{
    return (strcmp (ptr1->docno, ptr2->docno));
}

static int
parse_qrels_line (start_ptr, qid_ptr, docno_ptr, rel_ptr)
char **start_ptr, **qid_ptr, **docno_ptr, **rel_ptr;
{
    char *ptr = *start_ptr;

    /* Get qid */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    *qid_ptr = ptr;
    while (! isspace (*ptr)) ptr++;
    if (*ptr == '\n')  return (UNDEF);
    *ptr++ = '\0';
    /* Skip iter */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    while (! isspace (*ptr)) ptr++;
    if (*ptr++ == '\n') return (UNDEF);
    /* Get docno */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    *docno_ptr = ptr;
    while (! isspace (*ptr)) ptr++;
    if (*ptr == '\n') return (UNDEF);
    *ptr++ = '\0';
    /* Get relevance */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    if (*ptr == '\n') return (UNDEF);
    *rel_ptr = ptr;
    while (! isspace (*ptr)) ptr++;
    if (*ptr != '\n') {
	*ptr++ = '\0';
	while (*ptr != '\n' && isspace (*ptr)) ptr++;
	if (*ptr != '\n') return (UNDEF);
    }
    *ptr++ = '\0';
    *start_ptr = ptr;
    return (0);
}
