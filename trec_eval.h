#ifndef TRECEVALH
#define TRECEVALH

/* Static state info; set at beginning, possibly from program options, */
/* but then remains constant throughout. */
/* List of command line arguments giving individual measure parameters.
   meas_arg is NULL if there are no such arguments.  
   If arguments, final list member contains a NULL measure_name */
typedef struct {
	char *measure_name;
	char *parameters;
} MEAS_ARG;
    
typedef struct {
    long query_flag;              /* 0. If set, evaluation output will be
                                     printed for each query, in addition
                                     to summary at end. */
    long summary_flag;            /* 1. If set, evaluation output will be
                                     printed for summary */
    long debug_level;             /* 0. if level is 1 or 2, measure debug info
				     printed. If 3 or more, other info may
				     be printed (file format etc). */
    char *debug_query;            /* NULL. if non-NULL then only debug_query
				     will be evaluated */
    long relation_flag;           /* 1. If set, print in relational form */
    long average_complete_flag;   /* 0. If set, average over the complete set
				     of relevance judgements (qrels), instead
				     of the number of queries 
				     in the intersection of qrels and result */
    long judged_docs_only_flag;   /* 0. If set, throw out all unjudged docs
				     for the retrieved set before calculating
				     any measures. */
    long num_docs_in_coll;        /* 0. number of docs in collection */
    long relevance_level;         /* 1. In relevance judgements, the level at
				     which a doc is considered relevant for
				     this evaluation */
    long max_num_docs_per_topic;  /* MAXLONG. evaluate only this many docs */
    char *rel_info_format;        /* "qrels", format of input rel_info_file */
    char *results_format;        /* "trec_results"  format of input results*/
    /* List of command line arguments giving individual measure parameters.
       meas_arg is NULL if there are no such arguments.  
       If arguments, final list member contains a NULL measure_name */
    MEAS_ARG *meas_arg;
} EPI;



typedef struct {
    char *name;                    /* Full measure name for a trec_eval value.
				      This includes root measure name, plus
				      any changes due to cutoffs, parameters */
    long print_flags;              /* Any print flags.  See below */
    double value;                  /* Actual value */
} TREC_EVAL_VALUE;

/* Print Flags.  Gives long vs double, and whether to print for query or
   summary. */
#define TE_MVALUE_PRINT_LONG       1
#define TE_MVALUE_NO_PRINT_Q       2
#define TE_MVALUE_NO_PRINT_SUMMARY 4

/* Evaluation values being calculated */
typedef struct {
    char  *qid;                     /* query id  */
    long num_queries;               /* Number of queries for this eval */
    long num_orig_queries;          /* Number of queries for this eval without
                                       missing values, if using trec_eval -c */
    TREC_EVAL_VALUE *values;       /* Actual measures and their values */
    long num_values;                /* Number of individual measures */
    long max_num_values;            /* Max number of measures space for */
} TREC_EVAL;


/* Definitions of internal formats for retrieval ranking and relevance info */
/* These are format independent definitions, which have a pointer
   to the format dependent definitions found in trec_format.h.
      eg (void *) results->q_results will point to an object of type
      TEXT_RESULTS INFO if results->ret_format is "trec_results".
      (void *) rel_info->q_rel_info wiil similarly point to type
      TEXT_QRELS_INFO if rel_info->rel_type is "qrels". */
typedef struct {
    char *qid;                      /* query id */
    char * ret_format;              /* format of retranked data.
				       eg, "trec_results" */
    void *q_results;               /* retrieval ranking for this qid */
} RESULTS;

typedef struct {                    /* Overall retrieved results */
    char *run_id;                   /* run id */
    long num_q_results;            /* Number of RESULTS queries */
    long max_num_q_results;        /* Num queries space reserved for*/
    RESULTS *results;             /* Array of RETRANK query results */
} ALL_RESULTS;

typedef struct {
    char *qid;                      /* query id */
    char *rel_format;               /* format of rel_info data.  Eg, "qrels" */
    void *q_rel_info;               /* relevance info for this qid */
} REL_INFO;

typedef struct {                    /* Overall relevance judgements */
    long num_q_rels;                /* Number of REL_INFO queries */
    long max_num_q_rels;            /* Num queries space reserved for */
    REL_INFO *rel_info;             /* Array of REL_INFO queries */
} ALL_REL_INFO;

/* Measure characteristics (how to print them, average them). */
/* List of measures is in measures.c */
typedef struct trec_meas {
    /* Nmae of measure (or root name of set of measures) */
    char *name;
    /* Full explanation of measure, printed upon help request */
    char *explanation;
    /* Store parameters for measure in meas_params. Reserve space in
       TREC_EVAL.values for results of measure. Store individual measure
       names (possibly altered by parameters) in TREC_EVAL.values and
       initialize value to 0.0.
       Set tm->eval_index to start of reserved space */
    int (* init_meas) (EPI *epi, struct trec_meas *tm, TREC_EVAL *eval);
    /* Calculate actual measure for single query */
    int (* calc_meas) (const EPI *epi, const REL_INFO *rel,
		       const RESULTS *results,  const struct trec_meas *tm,
		       TREC_EVAL *eval);
    /* Merge info for single query into summary info */
    int (* acc_meas) (const EPI *epi, const struct trec_meas *tm,
		      const TREC_EVAL *q_eval, TREC_EVAL *summ_eval);
    /* Calculate final averages (if needed)  from summary info */
    int (* calc_avg) (const EPI *epi, const struct trec_meas *tm,
		      TREC_EVAL *eval);

    /* Measure dependent parameters, defaults given here can normally be
       overridden from command line by init_meas procedure */
    void *meas_params;    
     /* Index within TREC_EVAL.values for values for measure.
	-1 indicates measure not to be calculated (default).
	-2 indicates measure to be calculated, but has not yet been initialized.
        Set in init_meas */
    long eval_index;
} TREC_MEAS;

/* Mapping of a nickname to a group of measures that it applies to.
   Eg "official".  See measures.c for values */
typedef struct {
    char *name;
    char **name_list;
} TREC_MEASURE_NICKNAMES;


/* File Formats for retrieval and rel info */
/* List of formats is in formats.c */
typedef struct {
    char *name;
    char *explanation;
    int (* get_file) (EPI *epi, char *text_qrels_file,
		      ALL_REL_INFO *all_rel_info);
} REL_INFO_FILE_FORMAT;
typedef struct {
    char *name;
    char *explanation;
    int (* get_file) (EPI *epi, char *text_results_file,
		      ALL_RESULTS *all_results);
} RESULTS_FILE_FORMAT;

/* Standard Parameter structures used for some measures in
   trec_meas.meas_params */
typedef struct {
    long num_params;
    long *param_values;
} LONG_PARAMS;
typedef struct {
    char *printable_params;       /* Desired printable version of params */
    long num_params;
    double *param_values;
} FLOAT_PARAMS;
typedef struct {
    char* name;
    double value;
} FLOAT_PARAM_PAIR;
typedef struct {
    char *printable_params;       /* Desired printable version of params */
    long num_params;             
    FLOAT_PARAM_PAIR *param_values;
} PAIR_PARAMS;

/* Macros for marking measures to be calculated in this invocation */
#define MARK_MEASURE(x)  x.eval_index = -2;
#define MEASURE_MARKED(x) (-2 == x.eval_index)
#define MEASURE_REQUESTED(x) (0 <= x.eval_index)

/* Non standard relevance values to be assigned retrieved docs */
#define RELVALUE_NONPOOL -1
#define RELVALUE_UNJUDGED -2


/* Defined constants - non=important but used in several places */
#define INIT_NUM_QUERIES 50
#define INIT_NUM_RESULTS 1000
#define INIT_NUM_RELS 2000
#define INIT_NUM_REL_LEVELS 5

#define MAX_LEN_QUERY 127
#define MAX_LEN_MEASURE 100

/* Defined epsilon constants for several measures */
#define MIN_GEO_MEAN .00001
#define INFAP_EPSILON .00001

#endif /* TRECEVALH */
