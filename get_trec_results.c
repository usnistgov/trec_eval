/* Copyright (c) 2003, 1991, 1990, 1984 Chris Buckley.  */


#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "trec_format.h"
#include <ctype.h>

/* Read all retrieved results information from trec_results_file.
Read text tuples from trec_results_file of the form
     030  Q0  ZF08-175-870  0   4238   prise1
     qid iter   docno      rank  sim   run_id
giving TREC document numbers (a string) retrieved by query qid 
(a string) with similarity sim (a float).  The other fields are ignored,
with the exception that the run_id field of the last line is kept and
output.  In particular, note that the rank field is ignored here;
internally ranks will be assigned by sorting by the sim field with ties 
broken determinstically (using docno).
Sim is assumed to be higher for the docs to be retrieved first.
File may contain no NULL characters.
Any field following run_id is ignored.
*/

static int
parse_results_line (char **start_ptr, char **qid_ptr, char **docno_ptr,
		    char **sim_ptr, char **run_id_ptr);

int 
te_get_trec_results (EPI *epi, char *trec_results_file,ALL_RESULTS *all_results)
{
    int fd;
    int size = 0;
    char *trec_results_buf;
    char *ptr;
    char *current_qid;
    char *qid_ptr, *docno_ptr, *sim_ptr;
    char *run_id_ptr = "";
    long i;
    TEXT_RESULTS_INFO *current_results = NULL;
    long line_num;

    /* Read entire file into memory */
    if (-1 == (fd = open (trec_results_file, 0)) ||
        -1 == (size = lseek (fd, 0L, 2)) ||
        NULL == (trec_results_buf = malloc ((unsigned) size+2)) ||
        -1 == lseek (fd, 0L, 0) ||
        size != read (fd, trec_results_buf, size) ||
	-1 == close (fd)) {
	fprintf (stderr,
		 "trec_eval.get_trec_results: Cannot read results file '%s'\n",
		 trec_results_file);
        return (UNDEF);
    }

    if (size == 0)
	return (0);

    current_qid = "";

    /* Initialize all_results */
    all_results->num_q_results = 0;
    all_results->max_num_q_results = INIT_NUM_QUERIES;
    if (NULL == (all_results->results = Malloc (INIT_NUM_QUERIES,
						  RESULTS)))
	return (UNDEF);

    /* Append ending newline if not present, Append NULL terminator */
    if (trec_results_buf[size-1] != '\n') {
	trec_results_buf[size] = '\n';
	size++;
    }
    trec_results_buf[size] = '\0';

    ptr = trec_results_buf;
    line_num = 0;
    while (*ptr) {
	line_num++;
	/* Get current line */
	/* Ignore blank lines (people seem to insist on them!) */
	while (*ptr && *ptr != '\n' && isspace (*ptr)) ptr++;
	if (*ptr == '\n') {
	    ptr++;
	    continue;
	}
	if (UNDEF == parse_results_line (&ptr, &qid_ptr, &docno_ptr,
					 &sim_ptr, &run_id_ptr)) {
	    fprintf (stderr, "trec_eval.get_trec_results: Malformed line %ld\n",
		     line_num);
	    return (UNDEF);
	}

	if (0 != strcmp (qid_ptr, current_qid)) {
	    /* Query has changed. Must check if new query or this is more
	       judgements for an old query */
	    current_qid = qid_ptr;
	    for (i = 0; i < all_results->num_q_results; i++) {
		if (0 == strcmp (qid_ptr, all_results->results[i].qid))
		    break;
	    }
	    if (i >= all_results->num_q_results) {
		/* New unseen query, add and initialize it */
		if (all_results->num_q_results >=
		    all_results->max_num_q_results) {
		    all_results->max_num_q_results *= 10;
		    if (NULL == (all_results->results = 
				 Realloc (all_results->results,
					  all_results->max_num_q_results,
					  RESULTS)))
			return (UNDEF);
		}
		all_results->results[i].qid = qid_ptr;
		all_results->results[i].ret_format = "trec_results";
		if (NULL == (current_results = 
			     Malloc (1, TEXT_RESULTS_INFO)))
		    return (UNDEF);
		all_results->results[i].q_results = (void *)current_results;
		current_results->num_text_results = 0;
		current_results->max_num_text_results = INIT_NUM_RESULTS;
		if (NULL == (current_results->text_results =
			     Malloc (INIT_NUM_RESULTS, TEXT_RESULTS)))
		    return (UNDEF);
		all_results->num_q_results++;
	    }
	    else {
		/* Old query, just switch current_results */
		current_results = (TEXT_RESULTS_INFO *)
		    all_results->results[i].q_results;
	    }
	}
	
	/* Add retrieval docno/sim to current query's list */
	if (current_results->num_text_results >= 
	    current_results->max_num_text_results) {
	    /* Need more space */
	    current_results->max_num_text_results *= 10;
	    if (NULL == (current_results->text_results = 
			 Realloc (current_results->text_results,
				  current_results->max_num_text_results,
				  TEXT_RESULTS)))
		return (UNDEF);
	}
	current_results->text_results[current_results->num_text_results].docno = docno_ptr;
	current_results->text_results[current_results->num_text_results].sim =
	    atof (sim_ptr);
	current_results->num_text_results++;
    }

    all_results->run_id = run_id_ptr;

    return (1);
}

static int
parse_results_line (start_ptr, qid_ptr, docno_ptr, sim_ptr, run_id_ptr)
char **start_ptr, **qid_ptr, **docno_ptr, **sim_ptr, **run_id_ptr;
{
    char *ptr = *start_ptr;

    /* Get qid */
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
    /* Skip rank */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    while (! isspace (*ptr)) ptr++;
    if (*ptr++ == '\n') return (UNDEF);
    /* Get sim */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    *sim_ptr = ptr;
    while (! isspace (*ptr)) ptr++;
    if (*ptr == '\n')return (UNDEF);
    *ptr++ = '\0';
    /* Get run_id */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    if (*ptr == '\n') return (UNDEF);
    *run_id_ptr = ptr;
    while (! isspace (*ptr)) ptr++;
    if (*ptr != '\n') {
	/* Skip over rest of line */
	*ptr++ = '\0';
	while (*ptr != '\n') ptr++;
    }
    *ptr++ = '\0';
    *start_ptr = ptr;
    return (0);
}


