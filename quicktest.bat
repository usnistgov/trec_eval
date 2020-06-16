REM See https://stackoverflow.com/a/734634 for how to exit on failure
trec_eval test/qrels.test test/results.test > out.test && fc out.test test/out.test
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -m all_trec test/qrels.test test/results.test > out.test.a && fc out.test.a test/out.test.a
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -m all_trec -q test/qrels.test test/results.test > out.test.aq && fc out.test.aq test/out.test.aq
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -m all_trec -q -c test/qrels.test test/results.trunc > out.test.aqc && fc out.test.aqc test/out.test.aqc
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -m all_trec -q -c -M100 test/qrels.test test/results.trunc > out.test.aqcM && fc out.test.aqcM test/out.test.aqcM
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -m all_trec -mrelstring.20 -q -l2 test/qrels.rel_level test/results.test> out.test.aql && fc out.test.aql test/out.test.aql
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -m all_prefs -q -R prefs test/prefs.test test/prefs.results.test > out.test.prefs && fc out.test.prefs test/out.test.prefs
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -m all_prefs -q -R qrels_prefs test/qrels.test test/results.test > out.test.qrels_prefs && fc out.test.qrels_prefs test/out.test.qrels_prefs
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -m qrels_jg -q -R qrels_jg  test/qrels.123 test/results.test > out.test.qrels_jg && fc out.test.qrels_jg test/out.test.qrels_jg
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -q -miprec_at_recall..10,.20,.25,.75,.50 -m P.5,7,3 -m recall.20,2000 -m Rprec_mult.5.0,0.2,0.35 -mutility.2,-1,0,0 -m 11pt_avg..25,.5,.75 -mndcg.1=3,2=9,4=4.5 -mndcg_cut.10,20,23.4 -msuccess.2,5,20 test/qrels.test test/results.test > out.test.meas_params && fc out.test.meas_params test/out.test.meas_params
if %errorlevel% neq 0 exit /b %errorlevel%
trec_eval -q -m all_trec -Z test/zscores_file test/qrels.test test/results.test > out.test.aqZ && fc out.test.aqZ test/out.test.aqZ
if %errorlevel% neq 0 exit /b %errorlevel%
del out.test out.test.*