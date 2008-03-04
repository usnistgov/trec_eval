#ifdef RCSID
static char rcsid[] = "$Header: /home/smart/release/src/libevaluate/trvec_trec_eval.c,v 11.0 1992/07/21 18:20:35 chrisb Exp chrisb $";
#endif

/* Copyright (c) 2008
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"

/* Procedures for initialization of several default formats of measure values.
   Used in init_meas procedure description of TREC_MEAS in trec_eval.h:
      typedef struct trec_meas {
         ...
        * Store parameters for measure in meas_params. Reserve space in
	TREC_EVAL.values for results of measure. Store individual measure
	names (possibly altered by parameters) in TREC_EVAL.values.
	Set tm->eval_index to start of reserved space *
         ...
      } TREC_MEAS;
   Measures are defined in measures.c.
*/

/* Static utility functions defined later */
static void *get_long_cutoffs (char *param_string);
static void *get_float_cutoffs (char *param_string);
static void *get_float_params (char *param_string);
static void *get_param_pairs (char *param_string);
static int comp_long ();
static int comp_float ();
static char *append_long (char *name, long value);
static char *append_float (char *name, double value);
static char *append_string (char *name, char *value);

/* ---------------- Init measure generic procedures -------------- */

/* Measure does not require initialization or storage (eg num_q) */
int 
te_init_meas_empty (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval)
{
    tm->eval_index = 0;
    return (1);
}


/* Measure is a single float measure with no parameters,
   always printed */
int 
te_init_meas_s_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval)
{
    /* Make sure enough space */
    if (NULL == (eval->values =
		 te_chk_and_realloc (eval->values, &eval->max_num_values,
				     eval->num_values + 1,
				     sizeof (TREC_EVAL_VALUE))))
	return (UNDEF);

    /* Set location of value of measure, zero it, and increment
       space used for values */
    tm->eval_index = eval->num_values;
    eval->values[tm->eval_index] = (TREC_EVAL_VALUE) {tm->name, 0, 0.0};
    eval->num_values++;
    return (1);
}

/* Measure is a single float measure with no parameters,
   printed in summary only */
int 
te_init_meas_s_float_summ (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval)
{
    /* Make sure enough space */
    if (NULL == (eval->values =
		 te_chk_and_realloc (eval->values, &eval->max_num_values,
				     eval->num_values + 1,
				     sizeof (TREC_EVAL_VALUE))))
	return (UNDEF);

    /* Set location of value of measure, zero it, and increment
       space used for values */
    tm->eval_index = eval->num_values;
    eval->values[tm->eval_index] = (TREC_EVAL_VALUE)
	{tm->name, TE_MVALUE_NO_PRINT_Q, 0.0};
    eval->num_values++;
    return (1);
}

/* Measure is a single long measure with no parameters.
   always printed */
int 
te_init_meas_s_long (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval)
{
    /* Make sure enough space */
    if (NULL == (eval->values =
		 te_chk_and_realloc (eval->values, &eval->max_num_values,
				     eval->num_values + 1,
				     sizeof (TREC_EVAL_VALUE))))
	return (UNDEF);

    /* Set location of value of measure, zero it, and increment
       space used for values */
    tm->eval_index = eval->num_values;
    eval->values[tm->eval_index] = (TREC_EVAL_VALUE)
	{tm->name, TE_MVALUE_PRINT_LONG, 0.0};
    eval->num_values++;
    return (1);
}

/* Measure is a single long measure with no parameters.
   printed only in summary */
int 
te_init_meas_s_long_summ (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval)
{
    /* Make sure enough space */
    if (NULL == (eval->values =
		 te_chk_and_realloc (eval->values, &eval->max_num_values,
				     eval->num_values + 1,
				     sizeof (TREC_EVAL_VALUE))))
	return (UNDEF);

    /* Set location of value of measure, zero it, and increment
       space used for values */
    tm->eval_index = eval->num_values;
    eval->values[tm->eval_index] = (TREC_EVAL_VALUE)
	{tm->name, TE_MVALUE_PRINT_LONG | TE_MVALUE_NO_PRINT_Q, 0.0};
    eval->num_values++;
    return (1);
}

/* Measure is a float array with long cutoffs */
int 
te_init_meas_a_float_cut_long (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval)
{
    LONG_PARAMS *cutoffs;
    long i;
    /* See if there are command line parameters for this measure.
       Use those if given, otherwise use default cutoffs */
    if (epi->meas_arg) {
	MEAS_ARG *meas_arg_ptr = epi->meas_arg;
	while (meas_arg_ptr->measure_name) {
	    if (0 == strcmp (meas_arg_ptr->measure_name, tm->name)) {
		if (NULL == (tm->meas_params =
			     get_long_cutoffs (meas_arg_ptr->parameters)))
		    return (UNDEF);
	    }
	    meas_arg_ptr++;
	}
    }
    cutoffs = (LONG_PARAMS *) tm->meas_params;

    /* Make sure enough space */
    if (NULL == (eval->values =
		 te_chk_and_realloc (eval->values, &eval->max_num_values,
				     eval->num_values + cutoffs->num_params,
				     sizeof (TREC_EVAL_VALUE))))
	return (UNDEF);

    /* Initialize full measure name and value for each cutoff */
    for (i = 0; i < cutoffs->num_params; i++) {
	eval->values[eval->num_values+i] = (TREC_EVAL_VALUE)
	    {append_long (tm->name, cutoffs->param_values[i]), 0, 0.0};
	if (NULL == eval->values[eval->num_values+i].name)
	    return (UNDEF);
    }

    /* Set location of value of measure, and increment space used for values */
    tm->eval_index = eval->num_values;
    eval->num_values += cutoffs->num_params;
    return (1);
}

/* Measure is a float array with float cutoffs */
int 
te_init_meas_a_float_cut_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval)
{
    FLOAT_PARAMS *cutoffs;
    long i;
    /* See if there are command line parameters for this measure.
       Use those if given, otherwise use default cutoffs */
    if (epi->meas_arg) {
	MEAS_ARG *meas_arg_ptr = epi->meas_arg;
	while (meas_arg_ptr->measure_name) {
	    if (0 == strcmp (meas_arg_ptr->measure_name, tm->name)) {
		if (NULL == (tm->meas_params =
			     get_float_cutoffs (meas_arg_ptr->parameters)))
		    return (UNDEF);
	    }
	    meas_arg_ptr++;
	}
    }
    cutoffs = (FLOAT_PARAMS *) tm->meas_params;

    /* Make sure enough space */
    if (NULL == (eval->values =
		 te_chk_and_realloc (eval->values, &eval->max_num_values,
				     eval->num_values + cutoffs->num_params,
				     sizeof (TREC_EVAL_VALUE))))
	return (UNDEF);

    /* Initialize full measure name and value for each cutoff */
    for (i = 0; i < cutoffs->num_params; i++) {
	eval->values[eval->num_values+i] = (TREC_EVAL_VALUE)
	    {append_float (tm->name, cutoffs->param_values[i]), 0, 0.0};
	if (NULL == eval->values[eval->num_values+i].name)
	    return (UNDEF);
    }

    /* Set location of value of measure, and increment space used for values */
    tm->eval_index = eval->num_values;
    eval->num_values += cutoffs->num_params;
    return (1);
}

/* Note difference between float cutoffs and float params is that
   cutoff values are unordered, assumed unique, and one eval value is
   allocated to each cutoff. Param values are ordered and can be anything,
   and number of params means nothing about number of values*/

/* Measure is a single float with float params */
int 
te_init_meas_s_float_p_float (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval)
{
    FLOAT_PARAMS *params;
    /* See if there are command line parameters for this measure.
       Use those if given, otherwise use default cutoffs */
    if (epi->meas_arg) {
	MEAS_ARG *meas_arg_ptr = epi->meas_arg;
	while (meas_arg_ptr->measure_name) {
	    if (0 == strcmp (meas_arg_ptr->measure_name, tm->name)) {
		if (NULL == (tm->meas_params =
			     get_float_params (meas_arg_ptr->parameters)))
		    return (UNDEF);
	    }
	    meas_arg_ptr++;
	}
    }
    params = (FLOAT_PARAMS *) tm->meas_params;

    /* Make sure enough space */
    if (NULL == (eval->values =
		 te_chk_and_realloc (eval->values, &eval->max_num_values,
				     eval->num_values + 1,
				     sizeof (TREC_EVAL_VALUE))))
	return (UNDEF);

    /* Set location of value of measure, zero it, and increment
       space used for values */
    tm->eval_index = eval->num_values;
    if (params) 
	eval->values[tm->eval_index] = (TREC_EVAL_VALUE)
	    {append_string(tm->name, params->printable_params), 0, 0.0};
    else
	eval->values[tm->eval_index] = (TREC_EVAL_VALUE) {tm->name, 0, 0.0};

    if (NULL == eval->values[eval->num_values].name)
	    return (UNDEF);
    eval->num_values++;
    return (1);
}

/* Measure is a single float with paired name=float params */
int
te_init_meas_s_float_p_pair (EPI *epi, TREC_MEAS *tm, TREC_EVAL *eval) 
{
    PAIR_PARAMS *params;
     if (epi->meas_arg) {
        MEAS_ARG *meas_arg_ptr = epi->meas_arg;
        while (meas_arg_ptr->measure_name) {
            if (0 == strcmp (meas_arg_ptr->measure_name, tm->name)) {
                if (NULL == (tm->meas_params =
                             get_param_pairs (meas_arg_ptr->parameters)))
                    return (UNDEF);
            }
            meas_arg_ptr++;
        }
    }
    params = (PAIR_PARAMS *) tm->meas_params;

    /* Make sure enough space */
    if (NULL == (eval->values =
		 te_chk_and_realloc (eval->values, &eval->max_num_values,
				     eval->num_values + 1,	
				     sizeof (TREC_EVAL_VALUE))))
	return (UNDEF);

    /* Set location of value of measure, zero it, and increment
       space used for values */
    tm->eval_index = eval->num_values;
    if (params) 
	eval->values[tm->eval_index] = (TREC_EVAL_VALUE)
	    {append_string(tm->name, params->printable_params), 0, 0.0};
    else
	eval->values[tm->eval_index] = (TREC_EVAL_VALUE) {tm->name, 0, 0.0};
    if (NULL == eval->values[eval->num_values].name)
	    return (UNDEF);
    eval->num_values++;
    return (1);
}


/* ----------------- Utility procedures for initialization -------------- */
static void *
get_long_cutoffs (char *param_string)
{
    long num_cutoffs;
    char *ptr, *start_ptr;
    LONG_PARAMS *cutoffs;
    long i;

    /* Count number of parameters in param_string (comma separated) */
    num_cutoffs = 1;
    for (ptr = param_string; *ptr; ptr++) {
	if (*ptr == ',') num_cutoffs++;
    }

    /* Reserve space for cutoffs */
    if (NULL == (cutoffs = Malloc (1, LONG_PARAMS)) ||
	NULL == (cutoffs->param_values = Malloc (num_cutoffs, long)))
	return ((void *) NULL);

    cutoffs->num_params = num_cutoffs;
    start_ptr = param_string;
    num_cutoffs = 0;
    for (ptr = param_string; *ptr; ptr++) {
	if (*ptr == ',') {
	    *ptr = '\0';
	    cutoffs->param_values[num_cutoffs++] = atol(start_ptr);
	    start_ptr = ptr+1;
	}
    }
    cutoffs->param_values[num_cutoffs++] = atol(start_ptr);

    /* Sort cutoffs in increasing order */
    qsort ((char *) cutoffs->param_values,
           (int) num_cutoffs,
           sizeof (long),
           comp_long);

    /* Sanity checking: cutoffs > 0 and non-duplicates */
    if (cutoffs->param_values[0] <= 0)
	return ((void *) NULL);
    for (i = 1; i < num_cutoffs; i++) {
	if (cutoffs->param_values[i-1] == cutoffs->param_values[i])
	    return ((void *) NULL);
    }

    return ((void *)cutoffs);
}

static void *
get_float_cutoffs (char *param_string)
{
    long num_cutoffs;
    char *ptr, *start_ptr;
    FLOAT_PARAMS *cutoffs;
    long i;

    /* Count number of parameters in param_string (comma separated) */
    num_cutoffs = 1;
    for (ptr = param_string; *ptr; ptr++) {
	if (*ptr == ',') num_cutoffs++;
    }

    /* Reserve space for cutoffs */
    if (NULL == (cutoffs = Malloc (1, FLOAT_PARAMS)) ||
	NULL == (cutoffs->param_values = Malloc (num_cutoffs, double)))
	return ((void *) NULL);

    cutoffs->num_params = num_cutoffs;
    start_ptr = param_string;
    num_cutoffs = 0;
    for (ptr = param_string; *ptr; ptr++) {
	if (*ptr == ',') {
	    *ptr = '\0';
	    cutoffs->param_values[num_cutoffs++] = atof(start_ptr);
	    start_ptr = ptr+1;
	}
    }
    cutoffs->param_values[num_cutoffs++] = atof(start_ptr);

    /* Sort cutoffs in increasing order */
    qsort ((char *) cutoffs->param_values,
           (int) num_cutoffs,
           sizeof (double),
           comp_float);

    /* Sanity checking: cutoffs non-duplicates */
    for (i = 1; i < num_cutoffs; i++) {
	if (cutoffs->param_values[i-1] == cutoffs->param_values[i])
	    return ((void *) NULL);
    }

    return ((void *)cutoffs);
}

static void *
get_float_params (char *param_string)
{
    long num_params;
    char *ptr, *start_ptr;
    FLOAT_PARAMS *params;

    /* Count number of parameters in param_string (comma separated) */
    num_params = 1;
    for (ptr = param_string; *ptr; ptr++) {
	if (*ptr == ',') num_params++;
    }

    /* Reserve space for params */
    if (NULL == (params = Malloc (1, FLOAT_PARAMS)) ||
	NULL == (params->printable_params =
		 Malloc (strlen(param_string)+1, char)) ||
	NULL == (params->param_values = Malloc (num_params, double)))
	return ((void *) NULL);

    (void) strncpy (params->printable_params,
		    param_string,
		    strlen(param_string)+1);

    params->num_params = num_params;
    start_ptr = param_string;
    num_params = 0;
    for (ptr = param_string; *ptr; ptr++) {
	if (*ptr == ',') {
	    *ptr = '\0';
	    params->param_values[num_params++] = atof(start_ptr);
	    start_ptr = ptr+1;
	}
    }
    params->param_values[num_params++] = atof(start_ptr);

    return ((void *)params);
}

/* Params are in comma separated form name=float. Eg, -m ndcg_p.1=4.0,2=8.0 */
static void *
get_param_pairs (char *param_string)
{
    long num_params;
    char last_seen;
    char *ptr, *start_ptr;
    PAIR_PARAMS *params;

    /* Count number of parameters in param_string (comma separated), all
       of form name=value.  Return error if not of right form */
    num_params = 1;
    last_seen = ',';
    for (ptr = param_string; *ptr; ptr++) {
        if (*ptr == ',') {
	    num_params++;
	    if (*ptr == last_seen)  /* Error */
		break;
	    last_seen = ',';
	}
	else if (*ptr == '=') {
	    if (*ptr == last_seen) {/* Flag as error */
		last_seen = ',';
		break;
	    }
	    last_seen = '=';
	}
    }
    if (last_seen != '=') {
	fprintf (stderr, "trec_eval: malformed pair parameters in '%s'\n",
		 param_string);
	return (NULL);
    }

    /* Reserve space for params */
    if (NULL == (params = Malloc (1, PAIR_PARAMS)) ||
	NULL == (params->printable_params =
		 Malloc (strlen(param_string)+1, char)) ||
        NULL == (params->param_values = Malloc (num_params, FLOAT_PARAM_PAIR)))
        return ((void *) NULL);

    (void) strncpy (params->printable_params,
		    param_string,
		    strlen(param_string)+1);

    params->num_params = num_params;
    start_ptr = param_string;
    num_params = 0;
    for (ptr = param_string; *ptr; ptr++) {
        if (*ptr == '=') {
            *ptr = '\0';
            params->param_values[num_params].name = start_ptr;
            start_ptr = ptr+1;
        }
        else if (*ptr == ',') {
            *ptr = '\0';
            params->param_values[num_params++].value = atof(start_ptr);
            start_ptr = ptr+1;
        }
    }
    params->param_values[num_params++].value = atof(start_ptr);

    return ((void *)params);
}

static int
comp_long (long *ptr1, long *ptr2)
{
    return (*ptr1 - *ptr2);
}
static int
comp_float (double *ptr1, double *ptr2)
{
    return (*ptr1 - *ptr2);
}

static char *
append_long (char *name, long value)
{
    long length_required = strlen(name) + 20 + 1;
    char *full_name;
    if (NULL == (full_name = Malloc (length_required, char)))
	return (NULL);
    snprintf (full_name, length_required, "%s%ld", name, value);
    return (full_name);
}

static char *
append_float (char *name, double value)
{
    long length_required = strlen(name) + 8 + 1;
    char *full_name;
    if (NULL == (full_name = Malloc (length_required, char)))
	return (NULL);
    snprintf (full_name, length_required, "%s.%3.2f", name, value);
    return (full_name);
}
static char *
append_string (char *name, char *value)
{
    long length_required;
    char *full_name;
    if (NULL == value)
	return (name);
    length_required = strlen(name) + strlen(value) + 2;
    if (NULL == (full_name = Malloc (length_required, char)))
	return (NULL);
    snprintf (full_name, length_required, "%s.%s", name, value);
    return (full_name);
}
