use super::{check_crate, check_environment, check_manifest, check_root};
use crate::gates::workspace::Crate;

fn at(rel: &str) -> Crate {
    Crate {
        name: "x".to_owned(),
        rel: rel.to_owned(),
        deps: vec![],
        lib: None,
        no_std: false,
    }
}

fn rules(rel: &str, src: &str) -> Vec<&'static str> {
    let file = format!("{rel}/src/lib.rs");
    check_crate(&at(rel), &[(file, src.to_owned())])
        .into_iter()
        .map(|f| f.rule)
        .collect()
}

fn manifest(src: &str) -> toml::Table {
    toml::from_str(src).unwrap()
}

const NONE: Vec<&str> = Vec::new();

#[test]
fn an_assert_in_production_code_is_reported_and_debug_assert_is_not() {
    assert_eq!(
        rules("platform/clock", "pub fn f(x: u8) { assert!(x > 0); }"),
        vec!["source-assert"]
    );
    assert_eq!(
        rules("platform/clock", "pub fn f(x: u8) { assert_eq!(x, 1); }"),
        vec!["source-assert"]
    );
    assert_eq!(
        rules("platform/clock", "pub fn f(x: u8) { debug_assert!(x > 0); }"),
        NONE
    );
}

#[test]
fn a_test_may_assert() {
    let src = "#[cfg(test)]\nfn check() { assert!(true); }\n";
    assert_eq!(rules("platform/clock", src), NONE);
    let file = (
        "platform/clock/src/lib/tests.rs".to_owned(),
        "fn t() { assert!(true); }".to_owned(),
    );
    assert_eq!(check_crate(&at("platform/clock"), &[file]), vec![]);
}

#[test]
fn a_domain_value_keeps_its_fields_private_and_its_invariant_behind_a_constructor() {
    let domain = "services/billing/domain";
    assert_eq!(rules(domain, "pub struct Email(pub String);"), vec!["domain-value"]);
    assert_eq!(
        rules(domain, "pub struct Money { pub cents: u64 }"),
        vec!["domain-value"]
    );
    assert_eq!(
        rules(
            domain,
            "pub struct Email(String);\nimpl From<String> for Email { fn from(s: String) -> Self { Self(s) } }"
        ),
        vec!["domain-value"]
    );
    assert_eq!(
        rules(
            domain,
            "pub struct Email(String);\nimpl From<&str> for Email { fn from(s: &str) -> Self { Self(s.into()) } }"
        ),
        vec!["domain-value"]
    );
    assert_eq!(
        rules(
            domain,
            "pub struct Wrapper(u8);\nimpl core::ops::Deref for Wrapper { type Target = u8; fn deref(&self) -> &u8 { &self.0 } }"
        ),
        vec!["domain-value"]
    );
    assert_eq!(
        rules(
            domain,
            "pub struct Email(String);\npub(crate) struct Draft { pub(crate) raw: String }"
        ),
        NONE
    );
    assert_eq!(
        rules("services/billing/application", "pub struct Command { pub id: String }"),
        NONE,
        "only the domain"
    );
}

#[test]
fn a_service_holds_no_mutable_global_state() {
    let infra = "services/billing/infrastructure";
    assert_eq!(
        rules(
            infra,
            "static COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);"
        ),
        vec!["mutable-static"]
    );
    assert_eq!(
        rules(
            infra,
            "static CACHE: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());"
        ),
        vec!["mutable-static"]
    );
    assert_eq!(
        rules(
            infra,
            "thread_local! { static SEEN: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }"
        ),
        vec!["mutable-static"]
    );
    assert_eq!(
        rules(infra, "static NAME: &str = \"billing\";"),
        NONE,
        "an immutable static is fine"
    );
    assert_eq!(
        rules(
            "platform/task",
            "static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);"
        ),
        NONE,
        "platform may"
    );
}

#[test]
fn the_application_reaches_files_network_processes_and_environment_only_through_ports() {
    let app = "services/billing/application";
    assert_eq!(
        rules(app, "pub fn f() { let _x = std::fs::read_to_string(\"x\"); }"),
        vec!["application-io"]
    );
    assert_eq!(
        rules(app, "use std::net::TcpStream;\npub struct S(TcpStream);"),
        vec!["application-io"]
    );
    assert_eq!(
        rules(app, "pub fn f() -> bool { std::env::args().count() > 1 }"),
        vec!["application-io"]
    );
    assert_eq!(
        rules(app, "pub fn f() { let _o = std::io::stdout(); }"),
        vec!["application-io"]
    );
    assert_eq!(rules(app, "use std::sync::Mutex;\npub struct S(Mutex<u8>);"), NONE);
    assert_eq!(
        rules(
            "services/billing/infrastructure",
            "pub fn f() { let _x = std::fs::read(\"x\"); }"
        ),
        NONE,
        "adapters do I/O"
    );
}

#[test]
fn a_module_never_points_at_a_file_of_its_own_choosing() {
    assert_eq!(
        rules("platform/clock", "#[path = \"other.rs\"]\nmod inner;"),
        vec!["path-attr"]
    );
}

#[test]
fn an_error_message_is_lowercase_and_unpunctuated() {
    let p = "platform/clock";
    assert_eq!(
        rules(
            p,
            "#[derive(Debug, thiserror::Error)]\npub enum E { #[error(\"Something failed\")] A }"
        ),
        vec!["error-message"]
    );
    assert_eq!(
        rules(
            p,
            "#[derive(Debug, thiserror::Error)]\npub enum E { #[error(\"something failed.\")] A }"
        ),
        vec!["error-message"]
    );
    assert_eq!(
        rules(
            p,
            "#[derive(Debug, thiserror::Error)]\npub enum E { #[error(\"HTTP request failed\")] A, #[error(\"{0} is taken\")] B(u8), #[error(transparent)] C(std::io::Error) }"
        ),
        NONE
    );
}

#[test]
fn a_log_line_carries_its_values_as_fields_not_inside_the_message() {
    let p = "platform/clock";
    assert_eq!(
        rules(p, "pub fn f(id: u8) { tracing::info!(\"user {id} logged in\"); }"),
        vec!["log-structured"]
    );
    assert_eq!(
        rules(p, "pub fn f(id: u8) { tracing::warn!(\"user {} logged in\", id); }"),
        vec!["log-structured"]
    );
    assert_eq!(
        rules(p, "pub fn f(id: u8) { tracing::info!(user = id, \"logged in\"); }"),
        NONE
    );
}

#[test]
fn nothing_is_re_exported_by_glob() {
    assert_eq!(
        rules("platform/clock", "mod inner { pub fn one() {} }\npub use inner::*;"),
        vec!["glob-reexport"]
    );
    assert_eq!(
        rules("platform/clock", "mod inner { pub fn one() {} }\npub use inner::one;"),
        NONE
    );
}

#[test]
fn no_profile_turns_overflow_checks_or_debug_assertions_off() {
    let c = at("platform/clock");
    let off = "[profile.dev]\noverflow-checks = false\n[profile.test]\ndebug-assertions = false\n";

    let found: Vec<&str> = check_manifest(&c, &manifest(off)).into_iter().map(|f| f.rule).collect();

    assert_eq!(found, vec!["profile-overflow", "profile-overflow"]);
    assert_eq!(check_manifest(&c, &manifest("[profile.dev]\nopt-level = 1\n")), vec![]);
}

#[test]
fn the_release_profile_keeps_overflow_checks_on() {
    assert_eq!(
        check_root(&manifest("[profile.release]\noverflow-checks = true\n")),
        vec![]
    );
    assert_eq!(check_root(&manifest("[profile.release]\nlto = true\n")).len(), 1);
    assert_eq!(
        check_root(&manifest("[profile.release]\noverflow-checks = false\n")).len(),
        1
    );
}

#[test]
fn build_flags_from_the_environment_are_refused() {
    let clean = check_environment(|_| None);
    let flagged = check_environment(|k| (k == "RUSTFLAGS").then(|| "-Ctarget-cpu=native".to_owned()));
    let empty = check_environment(|k| (k == "RUSTC_WRAPPER").then(String::new));

    assert_eq!(clean, vec![]);
    assert_eq!(
        flagged.iter().map(|f| f.rule).collect::<Vec<_>>(),
        vec!["env-build-flags"]
    );
    assert_eq!(empty, vec![], "an empty variable sets nothing");
}
