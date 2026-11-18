# A warning or error from reading an integer says where the value comes
# from: the option, the measure, or the qrels file, topic and document.
# Before, it named only the value. The exit statuses are unchanged.
. test/regress/lib.sh
D=test/regress/str-to-long-context

# expect_message TEXT ARGS...: standard error must contain TEXT.
expect_message() {
    text=$1
    shift
    "$TREC_EVAL" "$@" 2>&1 >/dev/null | grep -F -q "$text" ||
        fail "trec_eval $*: no message containing: $text"
}

w="Warning, value \`1.5' partially read as \`1'"
expect_message "$w (cutoff of measure 'P')." -m P.1.5 $D/qrels $D/run
expect_status 0 -m P.1.5 $D/qrels $D/run
expect_message "Warning, value \`0x' partially read as \`0' (option -D)." \
    -D 0x -m num_rel $D/qrels $D/run
expect_message "$w (qrels file '$D/qrels.partial', topic 1, document d1)." \
    -m num_rel $D/qrels.partial $D/run
expect_status 0 -m num_rel $D/qrels.partial $D/run
expect_message "$w (qrels file '$D/qrels.partial', topic 1, jg 0, document d1)." \
    -R qrels_jg -m map_avgjg $D/qrels.partial $D/run
expect_status 0 -R qrels_jg -m map_avgjg $D/qrels.partial $D/run

# Floating-point parameters: cutoffs, plain parameters and name=value
# pairs.
expect_message "Warning, value \`.25x' partially read as \`.25' (cutoff of measure 'iprec_at_recall')." \
    -m iprec_at_recall..25x $D/qrels $D/run
expect_message "Warning, value \`0.5x' partially read as \`0.5' (parameter of measure 'set_F')." \
    -m set_F.0.5x $D/qrels $D/run
expect_message "Warning, value \`2x' partially read as \`2' (value of '1' in '1=2x')." \
    -m ndcg.1=2x $D/qrels $D/run
expect_message "Warning, value \`0.5x' partially read as \`0.5' (value of 'p' in 'p=0.5x')." \
    -m rbp.p=0.5x $D/qrels $D/run
expect_status 0 -m set_F.0.5x $D/qrels $D/run

# Errors carry the same context, and keep their exit statuses.
expect_message "Error: value \`x' could not be parsed (option -D)." \
    -D x $D/qrels $D/run
expect_status 1 -D x $D/qrels $D/run
expect_message "Error: value \`x' could not be parsed (cutoff of measure 'P')." \
    -m P.x $D/qrels $D/run
expect_status 2 -m P.x $D/qrels $D/run
expect_message "Error: value \`x' could not be parsed (parameter of measure 'set_F')." \
    -m set_F.x $D/qrels $D/run
expect_status 2 -m set_F.x $D/qrels $D/run
expect_message "Error: value \`inf' is not a finite number (cutoff of measure 'iprec_at_recall')." \
    -m iprec_at_recall.inf $D/qrels $D/run
expect_status 2 -m iprec_at_recall.inf $D/qrels $D/run
exit 0
