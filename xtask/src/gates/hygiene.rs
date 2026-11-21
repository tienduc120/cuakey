//! Rules adopted from published Rust guides (ADR-0013): what a lint cannot say about this repository's layers.

use proc_macro2::{Span, TokenTree};
use syn::spanned::Spanned as _;
use syn::visit::{self, Visit};
use syn::{Fields, GenericArgument, PathArguments, Type, UseTree, Visibility};
use toml::{Table, Value};

use super::Finding;
use super::source::is_cfg_test;
use super::workspace::{Crate, Location};

const ASSERTS: [&str; 3] = ["assert", "assert_eq", "assert_ne"];
const TRACING: [&str; 6] = ["trace", "debug", "info", "warn", "error", "event"];
/// Types whose static holds state that changes; a `LazyLock` or `OnceLock` set once is a cache, not state.
const MUTABLE: [&str; 4] = ["Mutex", "RwLock", "Cell", "RefCell"];
const PRIMITIVE: [&str; 18] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64", "bool",
    "char", "String", "str",
];
const IO_MODULES: [&str; 4] = ["fs", "net", "process", "env"];
const IO_STREAMS: [&str; 3] = ["stdin", "stdout", "stderr"];
const BUILD_FLAGS: [&str; 3] = ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER"];

/// Source rules for one crate; `files` are (path relative to the repository, source) pairs.
pub(crate) fn check_crate(c: &Crate, files: &[(String, String)]) -> Vec<Finding> {
    let layer = match c.location() {
        Location::Layer { layer, .. } => Some(layer),
        Location::Platform | Location::Tool | Location::Unknown => None,
    };
    let mut v = Hygiene {
        layer,
        ..Hygiene::default()
    };
    for (path, src) in files {
        if path.ends_with("/tests.rs") || path.contains("/tests/") {
            continue;
        }
        // A file that does not parse is the source rules' finding, not this one's.
        let Ok(file) = syn::parse_file(src) else { continue };
        v.file.clone_from(path);
        v.visit_file(&file);
    }
    v.found
}

/// A member's manifest: no profile turns overflow checks or debug assertions off.
pub(crate) fn check_manifest(c: &Crate, manifest: &Table) -> Vec<Finding> {
    let path = format!("{}/Cargo.toml", c.rel);
    profiles_turned_off(manifest)
        .into_iter()
        .map(|message| Finding {
            path: path.clone(),
            rule: "profile-overflow",
            message,
        })
        .collect()
}

/// The root manifest: the same, and the release profile keeps overflow checks on.
pub(crate) fn check_root(manifest: &Table) -> Vec<Finding> {
    let mut messages = profiles_turned_off(manifest);
    let release = profile(manifest, "release")
        .and_then(|p| p.get("overflow-checks"))
        .and_then(Value::as_bool);
    if release != Some(true) {
        messages.push(
            "[profile.release] must set overflow-checks = true: an overflow stops the program instead of producing a \
             wrong value (Android; ANSSI LANG-ARITH)"
                .to_owned(),
        );
    }
    messages
        .into_iter()
        .map(|message| Finding {
            path: "Cargo.toml".to_owned(),
            rule: "profile-overflow",
            message,
        })
        .collect()
}

/// Build flags from the environment silently replace the ones this repository sets (ANSSI DENV-CARGO-ENVVARS).
pub(crate) fn check_environment(get: impl Fn(&str) -> Option<String>) -> Vec<Finding> {
    BUILD_FLAGS
        .into_iter()
        .filter(|name| get(name).is_some_and(|v| !v.is_empty()))
        .map(|name| Finding {
            path: "environment".to_owned(),
            rule: "env-build-flags",
            message: format!(
                "{name} is set: it silently replaces the flags in .cargo/config.toml and the profiles this repository \
                 relies on; unset it"
            ),
        })
        .collect()
}

/// The real process environment, for `check_environment`.
pub(crate) fn process_env(name: &str) -> Option<String> {
    #[expect(
        clippy::disallowed_methods,
        reason = "this gate exists to read the build environment"
    )]
    std::env::var(name).ok()
}

fn profile<'a>(manifest: &'a Table, name: &str) -> Option<&'a Table> {
    manifest
        .get("profile")
        .and_then(Value::as_table)
        .and_then(|p| p.get(name))
        .and_then(Value::as_table)
}

fn profiles_turned_off(manifest: &Table) -> Vec<String> {
    let mut off = Vec::new();
    for name in ["dev", "test"] {
        let Some(p) = profile(manifest, name) else { continue };
        for key in ["overflow-checks", "debug-assertions"] {
            if p.get(key).and_then(Value::as_bool) == Some(false) {
                off.push(format!(
                    "[profile.{name}] turns {key} off: tests must catch what release would get wrong (ANSSI \
                     DENV-CARGO-OPTS)"
                ));
            }
        }
    }
    off
}

#[derive(Default)]
struct Hygiene {
    layer: Option<String>,
    file: String,
    found: Vec<Finding>,
}

impl Hygiene {
    fn push(&mut self, span: Span, rule: &'static str, message: &str) {
        let path = format!("{}:{}", self.file, span.start().line);
        self.found.push(Finding {
            path,
            rule,
            message: message.to_owned(),
        });
    }

    fn in_layer(&self, name: &str) -> bool {
        self.layer.as_deref() == Some(name)
    }
}

impl<'ast> Visit<'ast> for Hygiene {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        if is_cfg_test(&i.attrs) {
            return;
        }
        if i.attrs.iter().any(|a| a.path().is_ident("path")) {
            let message = "a module lives in the file named after it; #[path] hides where it is (Rust Style Guide)";
            self.push(i.span(), "path-attr", message);
        }
        visit::visit_item_mod(self, i);
    }

    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        if !is_cfg_test(&i.attrs) {
            visit::visit_item_fn(self, i);
        }
    }

    fn visit_impl_item_fn(&mut self, i: &'ast syn::ImplItemFn) {
        if !is_cfg_test(&i.attrs) {
            visit::visit_impl_item_fn(self, i);
        }
    }

    fn visit_macro(&mut self, i: &'ast syn::Macro) {
        let name = i.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
        if ASSERTS.contains(&name.as_str()) {
            let message = "an assert panics in production; return an error, or use debug_assert! for a check that \
                           only tests need (ANSSI LANG-LIMIT-PANIC-SRC)";
            self.push(i.span(), "source-assert", message);
        }
        if TRACING.contains(&name.as_str()) && formats_its_message(i) {
            let message = "log values as fields, `info!(user = id, \"logged in\")`, not inside the message, so they \
                           stay searchable (Microsoft M-LOG-STRUCTURED)";
            self.push(i.span(), "log-structured", message);
        }
        if name == "thread_local" && self.layer.is_some() {
            self.push(i.span(), "mutable-static", MUTABLE_STATIC);
        }
        visit::visit_macro(self, i);
    }

    fn visit_item_static(&mut self, i: &'ast syn::ItemStatic) {
        if self.layer.is_some() && holds_mutable_state(&i.ty) {
            self.push(i.span(), "mutable-static", MUTABLE_STATIC);
        }
        visit::visit_item_static(self, i);
    }

    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        if matches!(i.vis, Visibility::Public(_)) && has_glob(&i.tree) {
            let message = "re-export items by name, so each has one path a reader can find (Microsoft \
                           M-NO-GLOB-REEXPORTS)";
            self.push(i.span(), "glob-reexport", message);
        }
        if self.in_layer("application") {
            let mut paths = Vec::new();
            use_paths(&i.tree, &mut Vec::new(), &mut paths);
            if paths.iter().any(|p| is_io(p)) {
                self.push(i.span(), "application-io", APPLICATION_IO);
            }
        }
        visit::visit_item_use(self, i);
    }

    fn visit_path(&mut self, i: &'ast syn::Path) {
        if self.in_layer("application") {
            let segments: Vec<String> = i.segments.iter().map(|s| s.ident.to_string()).collect();
            if is_io(&segments) {
                self.push(i.span(), "application-io", APPLICATION_IO);
            }
        }
        visit::visit_path(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        let public_field = match &i.fields {
            Fields::Named(f) => f.named.iter().any(|f| matches!(f.vis, Visibility::Public(_))),
            Fields::Unnamed(f) => f.unnamed.iter().any(|f| matches!(f.vis, Visibility::Public(_))),
            Fields::Unit => false,
        };
        if self.in_layer("domain") && public_field {
            self.push(i.span(), "domain-value", DOMAIN_VALUE);
        }
        visit::visit_item_struct(self, i);
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if self.in_layer("domain")
            && let Some((_, path, _)) = &i.trait_
            && (is_deref(path) || is_from_primitive(path))
        {
            self.push(i.span(), "domain-value", DOMAIN_VALUE);
        }
        visit::visit_item_impl(self, i);
    }

    fn visit_attribute(&mut self, i: &'ast syn::Attribute) {
        if i.path().is_ident("error")
            && let Ok(text) = i.parse_args::<syn::LitStr>()
            && !is_good_message(&text.value())
        {
            let message = "an error message starts lowercase (a leading acronym is fine) and has no trailing period, \
                           so it reads well inside another message (rust-lang C-GOOD-ERR)";
            self.push(i.span(), "error-message", message);
        }
        visit::visit_attribute(self, i);
    }
}

const MUTABLE_STATIC: &str = "no global state that changes: pass it in, or own it in the composition root (Microsoft \
                              M-AVOID-STATICS)";
const APPLICATION_IO: &str = "the application reaches files, the network, processes, and the environment only through \
                              a port that infrastructure implements (Microsoft M-MOCKABLE-SYSCALLS)";
const DOMAIN_VALUE: &str = "a domain value keeps its fields private and its invariant behind a fallible constructor: no \
                            pub field, no Deref, no From<primitive> (Microsoft M-STRONG-TYPES; rust-lang \
                            C-STRUCT-PRIVATE)";

/// Whether a tracing macro's message string has a `{}` placeholder (`{{` is a literal brace).
fn formats_its_message(m: &syn::Macro) -> bool {
    m.tokens.clone().into_iter().any(|t| {
        if let TokenTree::Literal(lit) = t {
            let text = lit.to_string();
            (text.starts_with('"') || text.starts_with('r')) && text.replace("{{", "").contains('{')
        } else {
            false
        }
    })
}

fn holds_mutable_state(ty: &Type) -> bool {
    struct Idents(Vec<String>);
    impl<'ast> Visit<'ast> for Idents {
        fn visit_path_segment(&mut self, i: &'ast syn::PathSegment) {
            self.0.push(i.ident.to_string());
            visit::visit_path_segment(self, i);
        }
    }
    let mut idents = Idents(Vec::new());
    idents.visit_type(ty);
    idents
        .0
        .iter()
        .any(|i| MUTABLE.contains(&i.as_str()) || i.starts_with("Atomic"))
}

fn has_glob(tree: &UseTree) -> bool {
    match tree {
        UseTree::Glob(_) => true,
        UseTree::Path(p) => has_glob(&p.tree),
        UseTree::Group(g) => g.items.iter().any(has_glob),
        UseTree::Name(_) | UseTree::Rename(_) => false,
    }
}

/// Every full path a `use` brings in; a glob counts as its module.
fn use_paths(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            use_paths(&p.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(n) => out.push(prefix.iter().cloned().chain([n.ident.to_string()]).collect()),
        UseTree::Rename(r) => out.push(prefix.iter().cloned().chain([r.ident.to_string()]).collect()),
        UseTree::Glob(_) => out.push(prefix.clone()),
        UseTree::Group(g) => g.items.iter().for_each(|t| use_paths(t, prefix, out)),
    }
}

fn is_io(path: &[String]) -> bool {
    match path {
        [std, module, ..] if std == "std" && IO_MODULES.contains(&module.as_str()) => true,
        [std, io, stream, ..] if std == "std" && io == "io" && IO_STREAMS.contains(&stream.as_str()) => true,
        _ => false,
    }
}

fn is_deref(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|s| s.ident == "Deref" || s.ident == "DerefMut")
}

fn is_from_primitive(path: &syn::Path) -> bool {
    let Some(last) = path.segments.last() else { return false };
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };
    last.ident == "From"
        && args
            .args
            .iter()
            .any(|a| matches!(a, GenericArgument::Type(t) if is_primitive(t)))
}

fn is_primitive(ty: &Type) -> bool {
    if let Type::Reference(r) = ty {
        return is_primitive(&r.elem);
    }
    if let Type::Path(p) = ty {
        return p
            .path
            .segments
            .last()
            .is_some_and(|s| PRIMITIVE.contains(&s.ident.to_string().as_str()));
    }
    false
}

fn is_good_message(text: &str) -> bool {
    let mut chars = text.chars();
    let starts_upper = chars.next().is_some_and(char::is_uppercase);
    let acronym = chars.next().is_some_and(char::is_uppercase);
    (acronym || !starts_upper) && !text.ends_with('.')
}

#[cfg(test)]
mod tests;
