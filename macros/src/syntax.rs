#[allow(unused_extern_crates)]
extern crate proc_macro;

use core::ops;
use proc_macro::TokenStream;

use indexmap::{IndexMap, IndexSet};
use proc_macro2::TokenStream as TokenStream2;
use syn::Ident;

use crate::syntax::ast::App;

mod accessors;
pub mod analyze;
pub mod ast;
mod check;
mod optimize;
mod parse;

/// An ordered map keyed by identifier
pub type Map<T> = IndexMap<Ident, T>;

/// An order set
pub type Set<T> = IndexSet<T>;

/// Immutable pointer
pub struct P<T> {
    ptr: Box<T>,
}

impl<T> P<T> {
    /// Boxes `x` making the value immutable
    pub fn new(x: T) -> P<T> {
        P { ptr: Box::new(x) }
    }
}

impl<T> ops::Deref for P<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.ptr
    }
}

/// Execution context
#[derive(Clone, Copy)]
pub enum Context<'a> {
    /// The `idle` context
    Idle,

    /// The `init`-ialization function
    Init,

    /// A software task: `#[task]`
    SoftwareTask(&'a Ident),

    /// A hardware task: `#[exception]` or `#[interrupt]`
    HardwareTask(&'a Ident),
}

impl<'a> Context<'a> {
    /// The identifier of this context
    pub fn ident(&self, app: &'a App) -> &'a Ident {
        match self {
            Context::HardwareTask(ident) => ident,
            Context::Idle => &app.idle.as_ref().unwrap().name,
            Context::Init => &app.init.name,
            Context::SoftwareTask(ident) => ident,
        }
    }

    /// Is this the `idle` context?
    pub fn is_idle(&self) -> bool {
        matches!(self, Context::Idle)
    }

    /// Is this the `init`-ialization context?
    pub fn is_init(&self) -> bool {
        matches!(self, Context::Init)
    }

    /// Whether this context runs only once
    pub fn runs_once(&self) -> bool {
        self.is_init() || self.is_idle()
    }

    /// Whether this context has shared resources
    pub fn has_shared_resources(&self, app: &App) -> bool {
        match *self {
            Context::HardwareTask(name) => {
                !app.hardware_tasks[name].args.shared_resources.is_empty()
            }
            Context::Idle => !app.idle.as_ref().unwrap().args.shared_resources.is_empty(),
            Context::Init => false,
            Context::SoftwareTask(name) => {
                !app.software_tasks[name].args.shared_resources.is_empty()
            }
        }
    }

    /// Whether this context has local resources
    pub fn has_local_resources(&self, app: &App) -> bool {
        match *self {
            Context::HardwareTask(name) => {
                !app.hardware_tasks[name].args.local_resources.is_empty()
            }
            Context::Idle => !app.idle.as_ref().unwrap().args.local_resources.is_empty(),
            Context::Init => !app.init.args.local_resources.is_empty(),
            Context::SoftwareTask(name) => {
                !app.software_tasks[name].args.local_resources.is_empty()
            }
        }
    }
}

/// Parser and optimizer configuration
#[derive(Default)]
#[non_exhaustive]
pub struct Settings {
    /// Whether to accept the `binds` argument in `#[task]` or not
    pub parse_binds: bool,
    /// Whether to parse `extern` interrupts (functions) or not
    pub parse_extern_interrupt: bool,
    /// Whether to "compress" priorities or not
    pub optimize_priorities: bool,
}

/// Parses the input of the `#[app]` attribute
pub fn parse(
    args: TokenStream,
    input: TokenStream,
    settings: Settings,
) -> Result<(ast::App, analyze::Analysis), syn::parse::Error> {
    parse2(args.into(), input.into(), settings)
}

/// `proc_macro2::TokenStream` version of `parse`
pub fn parse2(
    args: TokenStream2,
    input: TokenStream2,
    settings: Settings,
) -> Result<(ast::App, analyze::Analysis), syn::parse::Error> {
    let mut app = parse::app(args, input, &settings)?;
    check::app(&app)?;
    optimize::app(&mut app, &settings);

    match analyze::app(&app) {
        Err(e) => Err(e),
        // If no errors, return the app and analysis results
        Ok(analysis) => Ok((app, analysis)),
    }
}

enum Either<A, B> {
    Left(A),
    Right(B),
}
