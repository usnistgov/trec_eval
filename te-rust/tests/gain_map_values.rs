use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gain-map-values")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn te_rust(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .output()
        .expect("failed to run te-rust")
}

fn eval(extra: &[&str]) -> Output {
    let (qrels, run) = (fixture("graded.qrels"), fixture("graded.run"));
    let mut args = extra.to_vec();
    args.extend([qrels.as_str(), run.as_str()]);
    te_rust(&args)
}

/// The printed value of `measure` on the `all` row.
fn summary(output: &Output, measure: &str) -> String {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .find(|fields| fields.len() == 3 && fields[0] == measure && fields[1] == "all")
        .unwrap_or_else(|| panic!("no summary row for {measure}"))[2]
        .to_string()
}

fn assert_refused(extra: &[&str], message: &str) {
    let out = eval(extra);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{extra:?} was accepted");
    assert!(out.stdout.is_empty(), "{extra:?} printed output");
    assert!(stderr.contains(message), "{extra:?}: {stderr}");
}

#[test]
fn gain_that_is_not_a_finite_number_is_refused() {
    // trec_eval with #119 refuses `1=x` and `1=` ("Illegal parameter value"); before it, both
    // read as 0. te-rust used the default gains and printed 0.9212.
    assert_refused(
        &["-m", "ndcg.1=x"],
        "invalid gain 'x' for relevance level 1",
    );
    assert_refused(&["-m", "ndcg.1="], "invalid gain '' for relevance level 1");
    assert_refused(&["-m", "ndcg.1=nan"], "invalid gain 'nan'");
    assert_refused(&["-m", "ndcg.1=inf"], "invalid gain 'inf'");
    assert_refused(&["-m", "rbp.1=abc"], "invalid gain 'abc'");
    assert_refused(&["-m", "rbp_resid.1=x"], "invalid gain 'x'");
}

#[test]
fn level_that_is_not_an_integer_is_refused() {
    // trec_eval skips such a name without a message (its valid_int test in gain_init.c).
    for measure in ["ndcg", "G", "ndcg_rel", "Rndcg", "ndcg_p", "rbp"] {
        assert_refused(
            &["-m", &format!("{measure}.x=1")],
            "invalid relevance level 'x'",
        );
    }
    assert_refused(&["-m", "ndcg.1.5=2"], "invalid relevance level '1.5'");
    assert_refused(&["-m", "ndcg.=2"], "invalid relevance level ''");
    // `p` is a parameter of rbp and rbp_resid only.
    assert_refused(&["-m", "ndcg.p=1"], "invalid relevance level 'p'");
    assert_refused(&["-m", "rbp.p=0.5,q=1"], "invalid relevance level 'q'");
    assert_refused(&["-m", "rbp_resid.q=1"], "invalid relevance level 'q'");
}

#[test]
fn malformed_pairs_are_refused() {
    // trec_eval: "malformed pair parameters" for each.
    assert_refused(&["-m", "ndcg.5,5"], "expected key=value, got '5'");
    assert_refused(&["-m", "ndcg.1=2,"], "expected key=value, got ''");
    assert_refused(&["-m", "ndcg.1=2=3"], "invalid gain '2=3'");
}

#[test]
fn repeated_level_is_refused() {
    // te-rust kept the last gain, and `ndcg.1=2,1=5` printed 0.7519, the value at gain 5.
    // trec_eval can score the run and the ideal ranking with different gains for such a
    // level. On this fixture it prints 0.9072 for `ndcg.1=2,1=5`, the value at gain 2, and
    // 0.6048 for `ndcg.1=5,1=2`, the run at gain 2 over the ideal at gain 5.
    assert_refused(
        &["-m", "ndcg.1=2,1=5"],
        "relevance level 1 is given more than once",
    );
    assert_refused(
        &["-m", "ndcg.1=2,01=3"],
        "relevance level 1 is given more than once",
    );
    assert_refused(
        &["-m", "rbp.p=0.5,1=3,1=5"],
        "relevance level 1 is given more than once",
    );
}

#[test]
fn repeated_p_is_refused() {
    // Both tools used the last p, and `rbp.p=0.5,p=0.8` printed the value of `rbp.p=0.8`.
    assert_refused(
        &["-m", "rbp.p=0.5,p=0.8"],
        "parameter p is given more than once",
    );
    assert_refused(
        &["-m", "rbp_resid.p=0.5,p=0.8"],
        "parameter p is given more than once",
    );
}

#[test]
fn global_gains_follow_the_same_rules() {
    assert_refused(&["--global-gains", "1=x", "-m", "ndcg"], "invalid gain 'x'");
    assert_refused(
        &["--global-gains", "1=2,1=5", "-m", "ndcg"],
        "given more than once",
    );
    assert_eq!(
        summary(&eval(&["--global-gains", "1=3,2=9", "-m", "ndcg"]), "ndcg"),
        "0.6988"
    );
}

#[test]
fn legal_maps_are_unchanged() {
    // The ndcg and G values are trec_eval's, and the rbp and rbp_resid values are hand
    // calculations, because trec_eval's rbp can read uninitialised memory (#94).
    // ndcg.1=3,2=9 by hand: d (grade 3, gain 3), a (gain 9) and c (gain 3) are at ranks 1,
    // 3 and 5, so DCG = 3 + 9/2 + 3/log2(6) = 8.6606; the ideal order 9, 3, 3 gives
    // 9 + 3/log2(3) + 3/2 = 12.3928, and 8.6606 / 12.3928 = 0.6988.
    assert_eq!(summary(&eval(&["-m", "ndcg"]), "ndcg"), "0.9212");
    assert_eq!(
        summary(&eval(&["-m", "ndcg.1=3,2=9"]), "ndcg_1=3,2=9"),
        "0.6988"
    );
    assert_eq!(
        summary(&eval(&["-m", "ndcg.+1=3,02=9"]), "ndcg_+1=3,02=9"),
        "0.6988"
    );
    assert_eq!(summary(&eval(&["-m", "G.1=3,2=9"]), "G_1=3,2=9"), "0.4251");
    // rbp divides each gain by the highest, 9: (3/9 + 9/9 * 0.5^2 + 3/9 * 0.5^4) * 0.5 = 0.3021.
    assert_eq!(
        summary(&eval(&["-m", "rbp.p=0.5,1=3,2=9"]), "rbp_p=0.5,1=3,2=9"),
        "0.3021"
    );
    // rbp_resid reads only p, and accepts the gain map that rbp reads, as trec_eval does.
    // The unjudged x at rank 2 of 5 gives (1 - 0.5) * 0.5 + 0.5^5 = 0.28125, printed as 0.2812.
    assert_eq!(
        summary(&eval(&["-m", "rbp_resid.p=0.5,1=3"]), "rbp_resid_p=0.5,1=3"),
        "0.2812"
    );
}
