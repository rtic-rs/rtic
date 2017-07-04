#![feature(plugin_registrar)]
#![feature(proc_macro_internals)]
#![feature(rustc_private)]
#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate rustc_errors;
extern crate rustc_plugin;
extern crate syn;
extern crate syntax as rustc_syntax;

use proc_macro::TokenStream;
use rustc_errors::Handler;
use rustc_errors::emitter::ColorConfig;
use rustc_plugin::Registry;
use rustc_syntax::codemap::{CodeMap, FilePathMapping};
use rustc_syntax::ext::base::SyntaxExtension;
use rustc_syntax::parse::ParseSess;
use rustc_syntax::symbol::Symbol;
use rustc_syntax::tokenstream::TokenStream as TokenStream_;
use std::rc::Rc;
use std::str::FromStr;

mod check;
mod syntax;
mod trans;
mod util;

fn expand_rtfm(ts: TokenStream_) -> TokenStream_ {
    let input = format!("{}", ts);

    let app = syntax::parse::app(&input);
    let ceilings = util::compute_ceilings(&app);
    check::resources(&app.resources, &ceilings);

    let output = format!("{}", trans::app(&app, &ceilings));

    let mapping = FilePathMapping::empty();
    let codemap = Rc::new(CodeMap::new(mapping));

    let tty_handler = Handler::with_tty_emitter(
        ColorConfig::Auto,
        true,
        false,
        Some(codemap.clone()),
    );

    let sess = ParseSess::with_span_handler(tty_handler, codemap.clone());
    proc_macro::__internal::set_parse_sess(&sess, || {
        let ts = TokenStream::from_str(&output).unwrap();
        proc_macro::__internal::token_stream_inner(ts)
    })
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        Symbol::intern("rtfm"),
        SyntaxExtension::ProcMacro(Box::new(expand_rtfm)),
    );
}
