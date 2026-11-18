# Each message from a judgments reader or a ranking former starts with
# trec_eval.<name>, where <name>.c is the file that prints it.
. test/regress/lib.sh
D=test/regress/message-prefixes

# check TEXT ARGS...: the first line of standard error must start with TEXT.
check() {
    text=$1
    shift
    line=$("$TREC_EVAL" "$@" 2>&1 >/dev/null | head -n 1)
    case $line in
    "$text"*) ;;
    *) fail "trec_eval $*: '$line' does not start with '$text'" ;;
    esac
}

check "trec_eval.get_qrels_prefs: Cannot read prefs file '$D/missing'" \
    -R qrels_prefs -m prefs_num_prefs_poss $D/missing $D/results
check "trec_eval.get_qrels_jg: duplicate docs d1" \
    -R qrels_jg -m map_avgjg $D/qrels.dup $D/results
check "trec_eval.form_res_rels: rel_info format not qrels" \
    -R qrels_jg -m map $D/qrels $D/results
check "trec_eval.form_res_rels: duplicate docs d1" \
    -m map $D/qrels $D/results.dup
check "trec_eval.form_res_rels_jg: duplicate docs d1" \
    -R qrels_jg -m map_avgjg $D/qrels $D/results.dup
check "trec_eval.form_prefs_counts: prefs_info format not" \
    -m prefs_num_prefs_poss $D/qrels $D/results
exit 0
