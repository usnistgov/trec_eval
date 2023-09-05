/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"

/* Procedures for initializing gain values from command-line parameters */

static int valid_int(const char *str)
{
    // Handle negative numbers.
    //
    if (*str == '-' || *str == '+')
        ++str;

    // Handle empty string or just "-" or "+".
    //
    if (!*str)
        return 0;

    // Check for non-digit chars in the rest of the stirng.
    //
    while (*str) {
        if (!isdigit(*str))
            return 0;
        else
            ++str;
    }

    return 1;
}

int setup_gains(const TREC_MEAS * tm, const RES_RELS * res_rels, GAINS * gains)
{
    DOUBLE_PARAM_PAIR *pairs = NULL;
    long num_pairs = 0;
    long i, j;
    long num_gains;

    if (tm->meas_params) {
        pairs = (DOUBLE_PARAM_PAIR *) tm->meas_params->param_values;
        num_pairs = tm->meas_params->num_params;
    }

    if (NULL == (gains->rel_gains = Malloc(res_rels->num_rel_levels + num_pairs,
                                           REL_GAIN)))
        return (UNDEF);
    num_gains = 0;
    for (i = 0; i < num_pairs; i++) {
        if (!valid_int(pairs[i].name))
            continue;
        gains->rel_gains[num_gains].rel_level = atol(pairs[i].name);
        gains->rel_gains[num_gains].gain = (double) pairs[i].value;
        gains->rel_gains[num_gains].num_at_level = 0;
        num_gains++;
    }

    for (i = 0; i < res_rels->num_rel_levels; i++) {
        for (j = 0; j < num_gains && gains->rel_gains[j].rel_level != i; j++);
        if (j < num_gains)
            /* Was included in list of parameters. Update occurrence info */
            gains->rel_gains[j].num_at_level = res_rels->rel_levels[i];
        else {
            /* Not included in list of parameters. New gain level */
            gains->rel_gains[num_gains].rel_level = i;
            gains->rel_gains[num_gains].gain = (double) i;
            gains->rel_gains[num_gains].num_at_level = res_rels->rel_levels[i];
            num_gains++;
        }
    }

    /* Sort gains by increasing gain value */
    qsort((char *) gains->rel_gains,
          (int) num_gains, sizeof(REL_GAIN), comp_rel_gain);

    gains->total_num_at_levels = 0;
    for (i = 0; i < num_gains; i++)
        gains->total_num_at_levels += gains->rel_gains[i].num_at_level;

    gains->num_gains = num_gains;
    return (1);
}

int comp_rel_gain(REL_GAIN * ptr1, REL_GAIN * ptr2)
{
    return (ptr1->gain - ptr2->gain);
}

double get_gain(const long rel_level, const GAINS * gains)
{
    long i;
    for (i = 0; i < gains->num_gains; i++)
        if (rel_level == gains->rel_gains[i].rel_level)
            return (gains->rel_gains[i].gain);
    return (0.0);               /* Print Error ?? */
}
