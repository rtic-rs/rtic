use std::collections::{HashMap, HashSet};

use syn::Ident;
use quote::Tokens;

pub mod parse;

#[derive(Debug)]
pub struct App {
    pub device: Tokens,
    pub idle: Idle,
    pub init: Init,
    pub resources: Statics,
    pub tasks: Tasks,
}

#[derive(Debug)]
pub struct Init {
    pub path: Tokens,
}

#[derive(Debug)]
pub struct Idle {
    pub local: Statics,
    pub path: Tokens,
    pub resources: HashSet<Ident>,
}

#[derive(Debug)]
pub struct Task {
    pub kind: Kind,
    pub priority: u8,
    pub resources: HashSet<Ident>,
}

#[derive(Debug)]
pub enum Kind {
    Exception,
    Interrupt { enabled: bool },
}

// $ident: $ty = $expr;
#[derive(Debug)]
pub struct Resource {
    pub expr: Tokens,
    pub ty: Tokens,
}

pub type Statics = HashMap<Ident, Resource>;

pub type Tasks = HashMap<Ident, Task>;
