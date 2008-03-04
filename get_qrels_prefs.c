/* Copyright (c) 2008 Chris Buckley */

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "trec_format.h"
#include <ctype.h>


/* Exactly the same as get_prefs.c except it reads a standard
   TREC qrels file as if it were a prefs file, associating
   JG with the iter field, and assigning JSG a constant '0'
   for all lines.  This normally will yield a single JG and
   a single JSG per query, with two levels of relevance if
   it is a binary relevance file, and more if a multiple
   relevance file.

Read all relevance preference information from text_qrels_prefs_file.
Preferences of user(s) for docs for a given qid is determined from
text_prefs_file, which consists of text tuples of the form
   qid  ujg   docno  rel_level
giving TREC document numbers (docno, a string) and their relevance
level (rel_level,a non-negative float) to query qid (a string) for a 
 user judgment group (ujg, a string).
Fields are separated by whitespace, string fields can contain no whitespace.
File may contain no NULL characters.

Preferences are indicated indirectly by comparing rel_level of
different docnos within the same user judgment group(JG).  A
judgment group establishes preferences between all docnos with
non-tied rel_levels within the group. Except possibly for 0.0, the
actual values of rel_level are ignored by default; they only serve to
establish a ranking within the JSG.

If a user only expresses a preference between two docs, then that user JSG
will have 2 lines in text_prefs_file:
      qid1  ujg1   docno1  3.0
      qid1  ujg1   docno2  2.0

If a user completely ranks some small number N (5-10) of docs, then N lines 
are used.
For example:
      qid1  ujg1    docno1  3.0
      qid1  ujg1    docno2  2.0
      qid1  ujg1    docno3  0.0
      qid1  ujg1    docno4  6.0
      qid1  ujg1    docno5  0.0
      qid1  ujg1    docno6  2.0
establishes a total of 13 preferences (5 with docno4 preferred, 4 with docno1 
preferred, 2 each with docno2 and docno6 preferred).

A Judgment Group (JG) conceptually represents preferences for a single
information need of a user at a single time.  Within a single JG, it
is an error if there are inconsistencies (doc A > doc B in one JSG,
but B > A or B == A in another).

Different JGs may contain contradictory preferences, These
disagreements are realistic and desirable: users (or even the same
user at different times) often do not agree with each other's
preferences.  Individual preference evaluation measures will handle
these contradictions (or confirmations) in different ways.

A rel_level of 0.0 by convention means that doc is non-relevant to the
topic (in that user's opinion).  Some preference evaluation measures
may handle 0.0 differently.  Thus when converting a preference file in
some other format into text_prefs format, do not assign a rel_level of
0.0 to a docno unless it is known that docno was considered
nonrelevant.

Handling of rel_level 0.0 separately addresses the general problem
that the number of nonrelevant docs judged for a topic can be critical
to fair evaluation - adding a couple of hundred preferences involving
nonrelevant docs (out of the possibly millions or billions in a
collection) can both change the importance of the topic when averaging
and even change whether system A scores better than system B on a
topic (even given identical retrieval on the added nonrel docs).  How
to handle this correctly for preference evaluation will be an
important future research problem.

*/

static int parse_qrels_prefs_line (char **start_ptr, char **qid_ptr,
				   char **jg_ptr, char **jsg_ptr,
				   char **docno_ptr, char **rel_ptr);

int 
te_get_qrels_prefs (EPI *epi, char *text_prefs_file, ALL_REL_INFO *all_rel_info)
{
    int fd;
    int size = 0;
    char *trec_prefs_buf;
    char *ptr;
    char *current_qid;
    char *qid_ptr, *jg_ptr, *jsg_ptr, *docno_ptr, *rel_ptr;
    long i;
    TEXT_PREFS_INFO *current_prefs = NULL;
    long line_num;

    /* Read entire file into memory */
    if (-1 == (fd = open (text_prefs_file, 0)) ||
        -1 == (size = lseek (fd, 0L, 2)) ||
        NULL == (trec_prefs_buf = malloc ((unsigned) size+2)) ||
        -1 == lseek (fd, 0L, 0) ||
        size != read (fd, trec_prefs_buf, size) ||
	-1 == close (fd)) {
        fprintf (stderr, "trec_eval.get_prefs: Cannot read prefs file\n");
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
    if (trec_prefs_buf[size-1] != '\n') {
	trec_prefs_buf[size] = '\n';
	size++;
    }
    trec_prefs_buf[size] = '\0';

    ptr = trec_prefs_buf;
    line_num = 0;
    while (*ptr) {
	line_num++;
	/* Get current line */
	if (UNDEF == parse_qrels_prefs_line (&ptr, &qid_ptr, &jg_ptr, &jsg_ptr,
				       &docno_ptr, &rel_ptr)) {
	    fprintf (stderr, "trec_eval.get_prefs: Malformed line %ld\n",
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
		all_rel_info->rel_info[i].rel_format = "prefs";
		if (NULL == (all_rel_info->rel_info[i].q_rel_info = 
			     Malloc (1, TEXT_PREFS_INFO)))
		    return (UNDEF);
		current_prefs = all_rel_info->rel_info[i].q_rel_info;
		current_prefs->num_text_prefs = 0;
		current_prefs->max_num_text_prefs = INIT_NUM_RELS;
		if (NULL == (current_prefs->text_prefs =
			     Malloc (INIT_NUM_RELS, TEXT_PREFS)))
		    return (UNDEF);
		all_rel_info->num_q_rels++;
	    }
	    else {
		/* Old query, just switch current_prefs */
		current_prefs = (TEXT_PREFS_INFO *)
		    all_rel_info->rel_info[i].q_rel_info;
	    }
	}
	
	/* Add judgement to current query's list */
	if (current_prefs->num_text_prefs >= 
	    current_prefs->max_num_text_prefs) {
	    /* Need more space */
	    current_prefs->max_num_text_prefs *= 10;
	    if (NULL == (current_prefs->text_prefs = 
			 Realloc (current_prefs->text_prefs,
				  current_prefs->max_num_text_prefs,
				  TEXT_PREFS)))
		return (UNDEF);
	}
	current_prefs->text_prefs[current_prefs->num_text_prefs].jg = jg_ptr;
	current_prefs->text_prefs[current_prefs->num_text_prefs].docno =
		docno_ptr;
	current_prefs->text_prefs[current_prefs->num_text_prefs].jsg = jsg_ptr;
	current_prefs->text_prefs[current_prefs->num_text_prefs].rel_level =
	    atof (rel_ptr);
	current_prefs->num_text_prefs++;
    }

    if (epi->debug_level >= 5) {
	TEXT_PREFS_INFO *tpi;
	long i,j;
	char *qid;
	/* Dump prefs file */
	printf ("Prefs File Dump - %ld queries\n", all_rel_info->num_q_rels);
	for (i = 0; i < all_rel_info->num_q_rels; i++) {
	    qid = all_rel_info->rel_info[i].qid;
	    tpi = (TEXT_PREFS_INFO *) all_rel_info->rel_info[i].q_rel_info;
	    printf ("  QID '%s', format '%s', num_pref lines %ld\n",
		     qid, all_rel_info->rel_info[i].rel_format,
		     tpi->num_text_prefs);
	    for (j = 0; j < tpi->num_text_prefs; j++) {
		printf ("  %s\t%s\t%s\t%2.4f\t%s\n",
			qid, tpi->text_prefs[j].jg, tpi->text_prefs[j].jsg,
			tpi->text_prefs[j].rel_level, tpi->text_prefs[j].docno);
	    }
	}
	fflush (stdout);
    }

    return (1);
}

static int
parse_qrels_prefs_line (char **start_ptr, char **qid_ptr, char**jg_ptr,
		  char **jsg_ptr, char **docno_ptr, char **rel_ptr)
{
    char *ptr = *start_ptr;
    *jsg_ptr = "0";

    /* Get qid */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    *qid_ptr = ptr;
    while (! isspace (*ptr)) ptr++;
    if (*ptr == '\n')  return (UNDEF);
    *ptr++ = '\0';
    /* Get Judgment Group */
    while (*ptr != '\n' && isspace (*ptr)) ptr++;
    *jg_ptr = ptr;
    while (! isspace (*ptr)) ptr++;
    if (*ptr == '\n') return (UNDEF);
    *ptr++ = '\0';
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
