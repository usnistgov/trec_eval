BIN = /usr/local/bin
H   = .

VERSIONID = 9.0alpha.1

# gcc
CC       = gcc
CFLAGS   = -g -I$H  -Wall -DMDEBUG -DVERSIONID=\"$(VERSIONID)\"
CFLAGS   = -g -I$H -O3 -Wall -DVERSIONID=\"$(VERSIONID)\"
CFLAGS   = -g -I$H  -Wall -DVERSIONID=\"$(VERSIONID)\"

# Other macros used in some or all makefiles
INSTALL = /bin/mv

TOP_SRCS = trec_eval.c formats.c meas_init.c meas_acc.c meas_avg.c 

FORMAT_SRCS = get_qrels.c get_trec_results.c get_prefs.c get_qrels_prefs.c \
           form_ordered_rel.c form_prefs_counts.c utility_pool.c

MEAS_SRCS =  measures.c  m_map.c m_P.c m_num_q.c m_num_ret.c m_num_rel.c \
        m_num_rel_ret.c m_gm_ap.c m_Rprec.c m_recip_rank.c m_bpref.c \
	m_ircl_prn.c m_recall.c m_R-prec-at.c m_utility.c m_11-pt_avg.c \
        m_ndcg.c m_ndcg_at.c m_ndcg_p.c m_rel_P.c m_success.c \
        m_set_P.c m_set_recall.c m_set_rel_P.c \
	m_set_map.c m_set_F.c m_num_nonrel_judged_ret.c \
	m_prefs_num_prefs_poss.c m_prefs_num_prefs_ful.c \
        m_prefs_num_prefs_ful_ret.c\
	m_prefs_simp.c m_prefs_pair.c m_prefs_avgjg.c m_prefs_avgjg_Rnonrel.c \
	m_prefs_simp_ret.c m_prefs_pair_ret.c m_prefs_avgjg_ret.c\
        m_prefs_avgjg_Rnonrel_ret.c \
	m_prefs_simp_imp.c m_prefs_pair_imp.c m_prefs_avgjg_imp.c\

SRCS = $(TOP_SRCS) $(FORMAT_SRCS) $(MEAS_SRCS)

SRCH = common.h trec_eval.h sysfunc.h functions.h trec_format.h

SRCOTHER = README Makefile test bpref_bug

trec_eval: $(SRCS) Makefile $(SRCH)
	$(CC) $(CFLAGS)  -o trec_eval $(SRCS) -lm

install: $(BIN)/trec_eval

quicktest: trec_eval
	./trec_eval test/qrels.test test/results.test | diff - test/out.test
	./trec_eval -m all_trec test/qrels.test test/results.test | diff - test/out.test.a
	./trec_eval -m all_trec -q test/qrels.test test/results.test | diff - test/out.test.aq
	./trec_eval -m all_trec -q -c test/qrels.test test/results.trunc | diff - test/out.test.aqc
	./trec_eval -m all_trec -q -c -M100 test/qrels.test test/results.trunc | diff - test/out.test.aqcM
	./trec_eval -m all_trec -q -l2 test/qrels.rel_level test/results.test | diff - test/out.test.aql
	./trec_eval -m all_prefs -q -R prefs test/prefs.test test/prefs.results.test | diff - test/out.test.prefs
	./trec_eval -m all_prefs -q -R qrels_prefs test/qrels.test test/results.test | diff - test/out.test.qrels_prefs

	/bin/echo "Test succeeeded"

longtest: trec_eval
	/bin/rm -rf test.long; mkdir test.long
	./trec_eval test/qrels.test test/results.test > test.long/out.test
	./trec_eval -m all_trec test/qrels.test test/results.test > test.long/out.test.a
	./trec_eval -m all_trec -q test/qrels.test test/results.test > test.long/out.test.aq
	./trec_eval -m all_trec -q -c test/qrels.test test/results.trunc > test.long/out.test.aqc
	./trec_eval -m all_trec -q -c -M100 test/qrels.test test/results.trunc > test.long/out.test.aqcM
	./trec_eval -m all_trec -q -l2 test/qrels.rel_level test/results.test > test.long/out.test.aql
	./trec_eval -m all_prefs -q -R prefs test/prefs.test test/prefs.results.test > test.long/out.test.prefs
	./trec_eval -m all_prefs -q -R qrels_prefs test/qrels.test test/results.test > test.long/out.test.qrels_prefs
	diff test.long test

$(BIN)/trec_eval: trec_eval
	if [ -f $@ ]; then $(INSTALL) $@ $@.old; fi;
	$(INSTALL) trec_eval $@

##4##########################################################################
##5##########################################################################
#  All code below this line (except for automatically created dependencies)
#  is independent of this particular makefile, and should not be changed!
#############################################################################

#########################################################################
# Odds and ends                                                         #
#########################################################################
clean semiclean:
	/bin/rm -f *.o *.BAK *~ trec_eval trec_eval.*.tar out.trec_eval Makefile.bak

tar:
	-/bin/rm -rf ./trec_eval.$(VERSIONID)
	mkdir trec_eval.$(VERSIONID)
	cp -rp $(SRCOTHER) $(SRCS) $(SRCH) trec_eval.$(VERSIONID)
	tar cf - ./trec_eval.$(VERSIONID) > trec_eval.$(VERSIONID).tar

#########################################################################
# Determining program dependencies                                      #
#########################################################################
depend:
	grep '^#[ ]*include' *.c \
		| sed -e 's?:[^"]*"\([^"]*\)".*?: \$H/\1?' \
			-e '/</d' \
			-e '/functions.h/d' \
		        -e 's/\.c/.o/' \
		        -e 's/\.y/.o/' \
		        -e 's/\.l/.o/' \
		> makedep
	echo '/^# DO NOT DELETE THIS LINE/+2,$$d' >eddep
	echo '$$r makedep' >>eddep
	echo 'w' >>eddep
	cp Makefile Makefile.bak
	ed - Makefile < eddep
	/bin/rm eddep makedep
	echo '# DEPENDENCIES MUST END AT END OF FILE' >> Makefile
	echo '# IF YOU PUT STUFF HERE IT WILL GO AWAY' >> Makefile
	echo '# see make depend above' >> Makefile

# DO NOT DELETE THIS LINE -- make depend uses it
# DEPENDENCIES MUST END AT END OF FILE
# IF YOU PUT STUFF HERE IT WILL GO AWAY
# see make depend above
