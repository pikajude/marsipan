extern crate rustc_version_runtime;

use commands::prelude::*;

pub fn about(e: Event) -> Hooks {
    e.respond(format!("\u{1f370} <b>marsipan v{}</b> built with rustc-{}",
        env!("CARGO_PKG_VERSION"),
        rustc_version_runtime::version()));
    vec![]
}
