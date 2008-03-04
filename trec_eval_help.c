/* Copyright (c) 2003, 1991, 1990, 1984 - Chris Buckley. */

#include "common.h"
#include "trec_eval.h"

static char *help_message = 
"trec_eval [-h] [-q] [-a] [-o] [-c] [-l<num>  [-N<num>] [-M<num>] [-Ua<num>] [-Ub<num>] [-Uc<num>] [-Ud<num>] [-T] trec_rel_file trec_top_file

Calculate and print various evaluation measures, evaluating the results 
in trec_top_file against the relevance judgements in trec_rel_file.

There are a fair number of options, of which only the lower case options are
normally ever used.  
 -h: Print full help message and exit
 -q: In addition to summary evaluation, give evaluation for each query
 -a: Print all evaluation measures calculated, instead of just the
     official measures for TREC 2.
 -o: Print everything out in old, nonrelational format (default is relational)
 -c: Average over the complete set of queries in the relevance judgements 
      instead of the queries in the intersection of relevance judgements
      and results.  Missing queries will contribute a value of 0 to all
      evaluation measures (which may or may not be reasonable for a 
      particular evaluation measure, but is reasonable for standard TREC
      measures.)
 -l<num>: Num indicates the minimum relevance judgement value needed for
      a document to be called relevant. (All measures used by TREC eval are
      based on binary relevance).  Used if trec_rel_file contains relevance
      judged on a multi-relevance scale.  Default is 1.
 -N<num>: Number of docs in collection
 -M<num>: Max number of docs per topic to use in evaluation (discard rest).
 -Ua<num>: Value to use for 'a' coefficient of utility computation.
                        relevant  nonrelevant
      retrieved            a          b
      nonretrieved         c          d
 -Ub<num>: Value to use for 'b' coefficient of utility computation.
 -Uc<num>: Value to use for 'c' coefficient of utility computation.
 -Ud<num>: Value to use for 'd' coefficient of utility computation.
 -J: Calculate all values only over the judged (either relevant or 
     nonrelevant) documents.  All unjudged documents are removed from the
     retrieved set before any calculations (possibly leaving an empty set).
     DO NOT USE, unless you really know what you're doing - very easy to get
     reasonable looking, but invalid, numbers. 
 -T: Treat similarity as time that document retrieved.  Compute
      several time-based measures after ranking docs by time retrieved
      (first doc (lowest sim) retrieved ranked highest). 
      Only done if -a selected.


Read text tuples from trec_top_file of the form
     030  Q0  ZF08-175-870  0   4238   prise1
     qid iter   docno      rank  sim   run_id
giving TREC document numbers (a string) retrieved by query qid 
(a string) with similarity sim (a float).  The other fields are ignored,
with the exception that the run_id field of the last line is kept and
output.  In particular, note that the rank field is ignored here;
internally ranks are assigned by sorting by the sim field with ties 
broken deterministicly (using docno).
Sim is assumed to be higher for the docs to be retrieved first.
File may contain no NULL characters.
Lines may contain fields after the run_id; they are ignored.

Relevance for each docno to qid is determined from text_qrels_file, which
consists of text tuples of the form
   qid  iter  docno  rel
giving TREC document numbers (docno, a string) and their relevance (rel, 
an integer) to query qid (a string).  iter string field is ignored.  
Fields are separated by whitespace, string fields can contain no whitespace.
File may contain no NULL characters.

The text tuples with relevance judgements are converted to TR_VEC form
and then submitted to the SMART evaluation routines.
The qid,did,rank,sim,rel fields of TR_VEC are filled in;
action,iter fields are set to 0.
The rel field is set to -1 if the document was not judged (not in 
text_qrels_file).  Most measures, but not all, will treat -1 the same as 0,
namely nonrelevant.  Note that relevance_level is used to determine if the
document is relevant during score calculations.
Queries for which there are no relevant docs are ignored.
Warning: queries for which there are relevant docs but no retrieved docs
are also ignored by default.  This allows systems to evaluate over subsets 
of the relevant docs, but means if a system improperly retrieves no docs, 
it will not be detected.  Use the -c flag to avoid this behavior.

EXPLANATION OF OFFICIAL VALUES PRINTED OF OLD NON-RELATIONAL FORMAT.
Relational Format prints the same values, but all lines are of the form
    measure_name   query   value

1. Total number of documents over all queries
        Retrieved:
        Relevant:
        Rel_ret:     (relevant and retrieved)
   These should be self-explanatory.  All values are totals over all
   queries being evaluated.
2. Interpolated Recall - Precision Averages:
        at 0.00
        at 0.10
        ...
        at 1.00
   See any standard IR text (especially by Salton) for more details of 
   recall-precision evaluation.  Measures precision (percent of retrieved
   docs that are relevant) at various recall levels (after a certain
   percentage of all the relevant docs for that query have been retrieved).
   'Interpolated' means that, for example, precision at recall
   0.10 (ie, after 10% of rel docs for a query have been retrieved) is
   taken to be MAXIMUM of precision at all recall points >= 0.10.
   Values are averaged over all queries (for each of the 11 recall levels).
   These values are used for Recall-Precision graphs.
3. Average precision (non-interpolated) over all rel docs
   The precision is calculated after each relevant doc is retrieved.
   If a relevant doc is not retrieved, its precision is 0.0.
   All precision values are then averaged together to get a single number
   for the performance of a query.  Conceptually this is the area
   underneath the recall-precision graph for the query.
   The values are then averaged over all queries.
4. Precision:
       at 5    docs
       at 10   docs
       ...
       at 1000 docs   
   The precision (percent of retrieved docs that are relevant) after X
   documents (whether relevant or nonrelevant) have been retrieved.
   Values averaged over all queries.  If X docs were not retrieved
   for a query, then all missing docs are assumed to be non-relevant.
5. R-Precision (precision after R (= num_rel for a query) docs retrieved):
   Measures precision (or recall, they're the same) after R docs
   have been retrieved, where R is the total number of relevant docs
   for a query.  Thus if a query has 40 relevant docs, then precision
   is measured after 40 docs, while if it has 600 relevant docs, precision
   is measured after 600 docs.  This avoids some of the averaging
   problems of the 'precision at X docs' values in (4) above.
   If R is greater than the number of docs retrieved for a query, then
   the nonretrieved docs are all assumed to be nonrelevant.
";


extern SINGLE_MEASURE sing_meas[];
extern PARAMETERIZED_MEASURE param_meas[];
extern int num_param_meas, num_sing_meas;

int
trec_eval_help(epi)
EVAL_PARAM_INFO *epi;
{
    long i, j;
    char temp_buf1[30];
    char temp_buf2[80];

    printf ("%s\n", help_message);

    printf ("Major measures (again) with their relational names:\n");
    for (i = 0; i < num_sing_meas; i++) {
	if (sing_meas[i].print_short_flag)
	    printf ("%-15s\t%s\n", sing_meas[i].name, sing_meas[i].long_name);
    }
    for (i = 0; i < num_param_meas; i++) {
	if (param_meas[i].print_short_flag) {
	    for (j = 0; j < param_meas[i].num_values; j++) {
		sprintf (temp_buf1, param_meas[i].format_string,
			 param_meas[i].get_param_str (epi, j));
		sprintf (temp_buf2, param_meas[i].long_format_string,
			 param_meas[i].get_param_str (epi, j));
		printf ("%-15s\t%s%s\n", temp_buf1,
			param_meas[i].long_name, temp_buf2);
	    }
	}
    }

    printf ("\n\nMinor measures with their relational names:\n");
    for (i = 0; i < num_sing_meas; i++) {
	if (sing_meas[i].print_short_flag)
	    continue;
	if (sing_meas[i].print_time_flag && (! epi->time_flag))
	    continue;
	if (! sing_meas[i].print_short_flag)
	    printf ("%-15s\t%s\n", sing_meas[i].name, sing_meas[i].long_name);
    }
    for (i = 0; i < num_param_meas; i++) {
	if (param_meas[i].print_short_flag)
	    continue;
	if (param_meas[i].print_time_flag && (! epi->time_flag))
	    continue;
	for (j = 0; j < param_meas[i].num_values; j++) {
	    sprintf (temp_buf1, param_meas[i].format_string,
		     param_meas[i].get_param_str (epi, j));
	    sprintf (temp_buf2, param_meas[i].long_format_string,
		     param_meas[i].get_param_str (epi, j));
	    printf ("%-15s\t%s%s\n", temp_buf1,
		    param_meas[i].long_name, temp_buf2);
	}
    }



    return (1);
}

