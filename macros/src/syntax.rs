use std::{
    collections::{HashMap, HashSet},
    iter, u8,
};

use proc_macro2::Span;
use syn::{
    braced, bracketed, parenthesized,
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Brace,
    ArgCaptured, AttrStyle, Attribute, Expr, FnArg, ForeignItem, Ident, IntSuffix, Item, ItemFn,
    ItemForeignMod, ItemStatic, LitInt, Path, PathArguments, PathSegment, ReturnType, Stmt, Token,
    Type, TypeTuple, Visibility,
};

pub struct AppArgs {
    pub device: Path,
}

impl Parse for AppArgs {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let mut device = None;
        loop {
            if input.is_empty() {
                break;
            }

            // #ident = ..
            let ident: Ident = input.parse()?;
            let _eq_token: Token![=] = input.parse()?;

            let ident_s = ident.to_string();
            match &*ident_s {
                "device" => {
                    if device.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    device = Some(input.parse()?);
                }
                _ => {
                    return Err(parse::Error::new(
                        ident.span(),
                        "expected `device`; other keys are not accepted",
                    ));
                }
            }

            if input.is_empty() {
                break;
            }

            // ,
            let _: Token![,] = input.parse()?;
        }

        Ok(AppArgs {
            device: device.ok_or(parse::Error::new(
                Span::call_site(),
                "`device` argument is required",
            ))?,
        })
    }
}

pub struct Input {
    _const_token: Token![const],
    _ident: Ident,
    _colon_token: Token![:],
    _ty: TypeTuple,
    _eq_token: Token![=],
    _brace_token: Brace,
    pub items: Vec<Item>,
    _semi_token: Token![;],
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        fn parse_items(input: ParseStream<'_>) -> parse::Result<Vec<Item>> {
            let mut items = vec![];

            while !input.is_empty() {
                items.push(input.parse()?);
            }

            Ok(items)
        }

        let content;
        Ok(Input {
            _const_token: input.parse()?,
            _ident: input.parse()?,
            _colon_token: input.parse()?,
            _ty: input.parse()?,
            _eq_token: input.parse()?,
            _brace_token: braced!(content in input),
            items: content.call(parse_items)?,
            _semi_token: input.parse()?,
        })
    }
}

pub struct App {
    pub args: AppArgs,
    pub idle: Option<Idle>,
    pub init: Init,
    pub exceptions: Exceptions,
    pub interrupts: Interrupts,
    pub resources: Resources,
    pub tasks: Tasks,
    pub free_interrupts: FreeInterrupts,
}

impl App {
    pub fn parse(items: Vec<Item>, args: AppArgs) -> parse::Result<Self> {
        let mut idle = None;
        let mut init = None;
        let mut exceptions = HashMap::new();
        let mut interrupts = HashMap::new();
        let mut resources = HashMap::new();
        let mut tasks = HashMap::new();
        let mut free_interrupts = None;

        for item in items {
            match item {
                Item::Fn(mut item) => {
                    if let Some(pos) = item.attrs.iter().position(|attr| eq(attr, "idle")) {
                        if idle.is_some() {
                            return Err(parse::Error::new(
                                item.span(),
                                "`#[idle]` function must appear at most once",
                            ));
                        }

                        let args = syn::parse2(item.attrs.swap_remove(pos).tts)?;

                        idle = Some(Idle::check(args, item)?);
                    } else if let Some(pos) = item.attrs.iter().position(|attr| eq(attr, "init")) {
                        if init.is_some() {
                            return Err(parse::Error::new(
                                item.span(),
                                "`#[init]` function must appear exactly once",
                            ));
                        }

                        let args = syn::parse2(item.attrs.swap_remove(pos).tts)?;

                        init = Some(Init::check(args, item)?);
                    } else if let Some(pos) =
                        item.attrs.iter().position(|attr| eq(attr, "exception"))
                    {
                        if exceptions.contains_key(&item.ident)
                            || interrupts.contains_key(&item.ident)
                            || tasks.contains_key(&item.ident)
                        {
                            return Err(parse::Error::new(
                                item.ident.span(),
                                "this task is defined multiple times",
                            ));
                        }

                        let args = syn::parse2(item.attrs.swap_remove(pos).tts)?;

                        exceptions.insert(item.ident.clone(), Exception::check(args, item)?);
                    } else if let Some(pos) =
                        item.attrs.iter().position(|attr| eq(attr, "interrupt"))
                    {
                        if exceptions.contains_key(&item.ident)
                            || interrupts.contains_key(&item.ident)
                            || tasks.contains_key(&item.ident)
                        {
                            return Err(parse::Error::new(
                                item.ident.span(),
                                "this task is defined multiple times",
                            ));
                        }

                        let args = syn::parse2(item.attrs.swap_remove(pos).tts)?;

                        interrupts.insert(item.ident.clone(), Interrupt::check(args, item)?);
                    } else if let Some(pos) = item.attrs.iter().position(|attr| eq(attr, "task")) {
                        if exceptions.contains_key(&item.ident)
                            || interrupts.contains_key(&item.ident)
                            || tasks.contains_key(&item.ident)
                        {
                            return Err(parse::Error::new(
                                item.ident.span(),
                                "this task is defined multiple times",
                            ));
                        }

                        let args = syn::parse2(item.attrs.swap_remove(pos).tts)?;

                        tasks.insert(item.ident.clone(), Task::check(args, item)?);
                    } else {
                        return Err(parse::Error::new(
                            item.span(),
                            "this item must live outside the `#[app]` module",
                        ));
                    }
                }
                Item::Static(item) => {
                    if resources.contains_key(&item.ident) {
                        return Err(parse::Error::new(
                            item.ident.span(),
                            "this resource is listed twice",
                        ));
                    }

                    resources.insert(item.ident.clone(), Resource::check(item)?);
                }
                Item::ForeignMod(item) => {
                    if free_interrupts.is_some() {
                        return Err(parse::Error::new(
                            item.abi.extern_token.span(),
                            "`extern` block can only appear at most once",
                        ));
                    }

                    free_interrupts = Some(FreeInterrupt::parse(item)?);
                }
                _ => {
                    return Err(parse::Error::new(
                        item.span(),
                        "this item must live outside the `#[app]` module",
                    ));
                }
            }
        }

        Ok(App {
            args,
            idle,
            init: init.ok_or_else(|| {
                parse::Error::new(Span::call_site(), "`#[init]` function is missing")
            })?,
            exceptions,
            interrupts,
            resources,
            tasks,
            free_interrupts: free_interrupts.unwrap_or_else(|| FreeInterrupts::new()),
        })
    }

    /// Returns an iterator over all resource accesses.
    ///
    /// Each resource access include the priority it's accessed at (`u8`) and the name of the
    /// resource (`Ident`). A resource may appear more than once in this iterator
    pub fn resource_accesses(&self) -> impl Iterator<Item = (u8, &Ident)> {
        self.idle
            .as_ref()
            .map(|idle| -> Box<dyn Iterator<Item = _>> {
                Box::new(idle.args.resources.iter().map(|res| (0, res)))
            })
            .unwrap_or_else(|| Box::new(iter::empty()))
            .chain(self.exceptions.values().flat_map(|e| {
                e.args
                    .resources
                    .iter()
                    .map(move |res| (e.args.priority, res))
            }))
            .chain(self.interrupts.values().flat_map(|i| {
                i.args
                    .resources
                    .iter()
                    .map(move |res| (i.args.priority, res))
            }))
            .chain(self.tasks.values().flat_map(|t| {
                t.args
                    .resources
                    .iter()
                    .map(move |res| (t.args.priority, res))
            }))
    }

    /// Returns an iterator over all `spawn` calls
    ///
    /// Each spawn call includes the priority of the task from which it's issued and the name of the
    /// task that's spawned. A task may appear more that once in this iterator.
    ///
    /// A priority of `None` means that this being called from `init`
    pub fn spawn_calls(&self) -> impl Iterator<Item = (Option<u8>, &Ident)> {
        self.init
            .args
            .spawn
            .iter()
            .map(|s| (None, s))
            .chain(
                self.idle
                    .as_ref()
                    .map(|idle| -> Box<dyn Iterator<Item = _>> {
                        Box::new(idle.args.spawn.iter().map(|s| (Some(0), s)))
                    })
                    .unwrap_or_else(|| Box::new(iter::empty())),
            )
            .chain(
                self.exceptions
                    .values()
                    .flat_map(|e| e.args.spawn.iter().map(move |s| (Some(e.args.priority), s))),
            )
            .chain(
                self.interrupts
                    .values()
                    .flat_map(|i| i.args.spawn.iter().map(move |s| (Some(i.args.priority), s))),
            )
            .chain(
                self.tasks
                    .values()
                    .flat_map(|t| t.args.spawn.iter().map(move |s| (Some(t.args.priority), s))),
            )
    }

    /// Returns an iterator over all `schedule` calls
    ///
    /// Each spawn call includes the priority of the task from which it's issued and the name of the
    /// task that's spawned. A task may appear more that once in this iterator.
    #[allow(dead_code)]
    pub fn schedule_calls(&self) -> impl Iterator<Item = (Option<u8>, &Ident)> {
        self.init
            .args
            .schedule
            .iter()
            .map(|s| (None, s))
            .chain(
                self.idle
                    .as_ref()
                    .map(|idle| -> Box<dyn Iterator<Item = _>> {
                        Box::new(idle.args.schedule.iter().map(|s| (Some(0), s)))
                    })
                    .unwrap_or_else(|| Box::new(iter::empty())),
            )
            .chain(self.exceptions.values().flat_map(|e| {
                e.args
                    .schedule
                    .iter()
                    .map(move |s| (Some(e.args.priority), s))
            }))
            .chain(self.interrupts.values().flat_map(|i| {
                i.args
                    .schedule
                    .iter()
                    .map(move |s| (Some(i.args.priority), s))
            }))
            .chain(self.tasks.values().flat_map(|t| {
                t.args
                    .schedule
                    .iter()
                    .map(move |s| (Some(t.args.priority), s))
            }))
    }

    #[allow(dead_code)]
    pub fn schedule_callers(&self) -> impl Iterator<Item = (Ident, &Idents)> {
        self.idle
            .as_ref()
            .map(|idle| -> Box<dyn Iterator<Item = _>> {
                Box::new(iter::once((
                    Ident::new("idle", Span::call_site()),
                    &idle.args.schedule,
                )))
            })
            .unwrap_or_else(|| Box::new(iter::empty()))
            .chain(iter::once((
                Ident::new("init", Span::call_site()),
                &self.init.args.schedule,
            )))
            .chain(
                self.exceptions
                    .iter()
                    .map(|(name, exception)| (name.clone(), &exception.args.schedule)),
            )
            .chain(
                self.interrupts
                    .iter()
                    .map(|(name, interrupt)| (name.clone(), &interrupt.args.schedule)),
            )
            .chain(
                self.tasks
                    .iter()
                    .map(|(name, task)| (name.clone(), &task.args.schedule)),
            )
    }

    pub fn spawn_callers(&self) -> impl Iterator<Item = (Ident, &Idents)> {
        self.idle
            .as_ref()
            .map(|idle| -> Box<dyn Iterator<Item = _>> {
                Box::new(iter::once((
                    Ident::new("idle", Span::call_site()),
                    &idle.args.spawn,
                )))
            })
            .unwrap_or_else(|| Box::new(iter::empty()))
            .chain(iter::once((
                Ident::new("init", Span::call_site()),
                &self.init.args.spawn,
            )))
            .chain(
                self.exceptions
                    .iter()
                    .map(|(name, exception)| (name.clone(), &exception.args.spawn)),
            )
            .chain(
                self.interrupts
                    .iter()
                    .map(|(name, interrupt)| (name.clone(), &interrupt.args.spawn)),
            )
            .chain(
                self.tasks
                    .iter()
                    .map(|(name, task)| (name.clone(), &task.args.spawn)),
            )
    }
}

pub type Idents = HashSet<Ident>;

pub type Exceptions = HashMap<Ident, Exception>;

pub type Interrupts = HashMap<Ident, Interrupt>;

pub type Resources = HashMap<Ident, Resource>;

pub type Statics = Vec<ItemStatic>;

pub type Tasks = HashMap<Ident, Task>;

pub type FreeInterrupts = HashMap<Ident, FreeInterrupt>;

pub struct Idle {
    pub args: IdleArgs,
    pub attrs: Vec<Attribute>,
    pub unsafety: Option<Token![unsafe]>,
    pub statics: HashMap<Ident, Static>,
    pub stmts: Vec<Stmt>,
}

pub type IdleArgs = InitArgs;

impl Idle {
    fn check(args: IdleArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = item.vis == Visibility::Inherited
            && item.constness.is_none()
            && item.asyncness.is_none()
            && item.abi.is_none()
            && item.decl.generics.params.is_empty()
            && item.decl.generics.where_clause.is_none()
            && item.decl.inputs.is_empty()
            && item.decl.variadic.is_none()
            && is_bottom(&item.decl.output);

        let span = item.span();

        if !valid_signature {
            return Err(parse::Error::new(
                span,
                "`idle` must have type signature `[unsafe] fn() -> !`",
            ));
        }

        let (statics, stmts) = extract_statics(item.block.stmts);

        Ok(Idle {
            args,
            attrs: item.attrs,
            unsafety: item.unsafety,
            statics: Static::parse(statics)?,
            stmts,
        })
    }
}

pub struct InitArgs {
    pub resources: Idents,
    pub schedule: Idents,
    pub spawn: Idents,
}

impl Default for InitArgs {
    fn default() -> Self {
        InitArgs {
            resources: Idents::new(),
            schedule: Idents::new(),
            spawn: Idents::new(),
        }
    }
}

impl Parse for InitArgs {
    fn parse(input: ParseStream<'_>) -> parse::Result<InitArgs> {
        if input.is_empty() {
            return Ok(InitArgs::default());
        }

        let mut resources = None;
        let mut schedule = None;
        let mut spawn = None;

        let content;
        parenthesized!(content in input);
        loop {
            if content.is_empty() {
                break;
            }

            // #ident = ..
            let ident: Ident = content.parse()?;
            let _: Token![=] = content.parse()?;

            let ident_s = ident.to_string();
            match &*ident_s {
                "schedule" if cfg!(not(feature = "timer-queue")) => {
                    return Err(parse::Error::new(
                        ident.span(),
                        "The `schedule` API requires that the `timer-queue` feature is \
                         enabled in the `cortex-m-rtfm` crate",
                    ));
                }
                "resources" | "schedule" | "spawn" => {} // OK
                _ => {
                    return Err(parse::Error::new(
                        ident.span(),
                        "expected one of: resources, schedule or spawn",
                    ));
                }
            }

            // .. [#(#idents)*]
            let inner;
            bracketed!(inner in content);
            let mut idents = Idents::new();
            for ident in inner.call(Punctuated::<_, Token![,]>::parse_terminated)? {
                if idents.contains(&ident) {
                    return Err(parse::Error::new(
                        ident.span(),
                        "element appears more than once in list",
                    ));
                }

                idents.insert(ident);
            }

            let ident_s = ident.to_string();
            match &*ident_s {
                "resources" => {
                    if resources.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    resources = Some(idents);
                }
                "schedule" => {
                    if schedule.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    schedule = Some(idents);
                }
                "spawn" => {
                    if spawn.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    spawn = Some(idents);
                }
                _ => unreachable!(),
            }

            if content.is_empty() {
                break;
            }

            // ,
            let _: Token![,] = content.parse()?;
        }

        Ok(InitArgs {
            resources: resources.unwrap_or(Idents::new()),
            schedule: schedule.unwrap_or(Idents::new()),
            spawn: spawn.unwrap_or(Idents::new()),
        })
    }
}

pub struct Assign {
    pub attrs: Vec<Attribute>,
    pub left: Ident,
    pub right: Box<Expr>,
}

pub struct Init {
    pub args: InitArgs,
    pub attrs: Vec<Attribute>,
    pub unsafety: Option<Token![unsafe]>,
    pub statics: HashMap<Ident, Static>,
    pub stmts: Vec<Stmt>,
    pub assigns: Vec<Assign>,
}

impl Init {
    fn check(args: InitArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = item.vis == Visibility::Inherited
            && item.constness.is_none()
            && item.asyncness.is_none()
            && item.abi.is_none()
            && item.decl.generics.params.is_empty()
            && item.decl.generics.where_clause.is_none()
            && item.decl.inputs.is_empty()
            && item.decl.variadic.is_none()
            && is_unit(&item.decl.output);

        let span = item.span();

        if !valid_signature {
            return Err(parse::Error::new(
                span,
                "`init` must have type signature `[unsafe] fn()`",
            ));
        }

        let (statics, stmts) = extract_statics(item.block.stmts);
        let (stmts, assigns) = extract_assignments(stmts);

        Ok(Init {
            args,
            attrs: item.attrs,
            unsafety: item.unsafety,
            statics: Static::parse(statics)?,
            stmts,
            assigns,
        })
    }
}

pub struct Exception {
    pub args: ExceptionArgs,
    pub attrs: Vec<Attribute>,
    pub unsafety: Option<Token![unsafe]>,
    pub statics: HashMap<Ident, Static>,
    pub stmts: Vec<Stmt>,
}

pub struct ExceptionArgs {
    pub priority: u8,
    pub resources: Idents,
    pub schedule: Idents,
    pub spawn: Idents,
}

impl Parse for ExceptionArgs {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        parse_args(input, false).map(
            |TaskArgs {
                 priority,
                 resources,
                 schedule,
                 spawn,
                 ..
             }| {
                ExceptionArgs {
                    priority,
                    resources,
                    schedule,
                    spawn,
                }
            },
        )
    }
}

impl Exception {
    fn check(args: ExceptionArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = item.vis == Visibility::Inherited
            && item.constness.is_none()
            && item.asyncness.is_none()
            && item.abi.is_none()
            && item.decl.generics.params.is_empty()
            && item.decl.generics.where_clause.is_none()
            && item.decl.inputs.is_empty()
            && item.decl.variadic.is_none()
            && is_unit(&item.decl.output);

        if !valid_signature {
            return Err(parse::Error::new(
                item.span(),
                "`exception` handlers must have type signature `[unsafe] fn()`",
            ));
        }

        let span = item.ident.span();
        match &*item.ident.to_string() {
            "MemoryManagement" | "BusFault" | "UsageFault" | "SecureFault" | "SVCall"
            | "DebugMonitor" | "PendSV" => {} // OK
            "SysTick" => {
                if cfg!(feature = "timer-queue") {
                    return Err(parse::Error::new(
                        span,
                        "the `SysTick` exception can't be used because it's used by \
                         the runtime when the `timer-queue` feature is enabled",
                    ));
                }
            }
            _ => {
                return Err(parse::Error::new(
                    span,
                    "only exceptions with configurable priority can be used as hardware tasks",
                ));
            }
        }

        let (statics, stmts) = extract_statics(item.block.stmts);

        Ok(Exception {
            args,
            attrs: item.attrs,
            unsafety: item.unsafety,
            statics: Static::parse(statics)?,
            stmts,
        })
    }
}

pub struct Interrupt {
    pub args: InterruptArgs,
    pub attrs: Vec<Attribute>,
    pub unsafety: Option<Token![unsafe]>,
    pub statics: HashMap<Ident, Static>,
    pub stmts: Vec<Stmt>,
}

pub type InterruptArgs = ExceptionArgs;

impl Interrupt {
    fn check(args: InterruptArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = item.vis == Visibility::Inherited
            && item.constness.is_none()
            && item.asyncness.is_none()
            && item.abi.is_none()
            && item.decl.generics.params.is_empty()
            && item.decl.generics.where_clause.is_none()
            && item.decl.inputs.is_empty()
            && item.decl.variadic.is_none()
            && is_unit(&item.decl.output);

        let span = item.span();

        if !valid_signature {
            return Err(parse::Error::new(
                span,
                "`interrupt` handlers must have type signature `[unsafe] fn()`",
            ));
        }

        match &*item.ident.to_string() {
            "init" | "idle" | "resources" => {
                return Err(parse::Error::new(
                    span,
                    "`interrupt` handlers can NOT be named `idle`, `init` or `resources`",
                ));
            }
            _ => {}
        }

        let (statics, stmts) = extract_statics(item.block.stmts);

        Ok(Interrupt {
            args,
            attrs: item.attrs,
            unsafety: item.unsafety,
            statics: Static::parse(statics)?,
            stmts,
        })
    }
}

pub struct Resource {
    pub singleton: bool,
    pub cfgs: Vec<Attribute>,
    pub attrs: Vec<Attribute>,
    pub mutability: Option<Token![mut]>,
    pub ty: Box<Type>,
    pub expr: Option<Box<Expr>>,
}

impl Resource {
    fn check(mut item: ItemStatic) -> parse::Result<Resource> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                item.span(),
                "resources must have inherited / private visibility",
            ));
        }

        let uninitialized = match *item.expr {
            Expr::Tuple(ref tuple) => tuple.elems.is_empty(),
            _ => false,
        };

        let pos = item.attrs.iter().position(|attr| eq(attr, "Singleton"));

        if let Some(pos) = pos {
            item.attrs[pos].path.segments.insert(
                0,
                PathSegment::from(Ident::new("owned_singleton", Span::call_site())),
            );
        }

        let (cfgs, attrs) = extract_cfgs(item.attrs);

        Ok(Resource {
            singleton: pos.is_some(),
            cfgs,
            attrs,
            mutability: item.mutability,
            ty: item.ty,
            expr: if uninitialized { None } else { Some(item.expr) },
        })
    }
}

pub struct TaskArgs {
    pub capacity: Option<u8>,
    pub priority: u8,
    pub resources: Idents,
    pub spawn: Idents,
    pub schedule: Idents,
}

impl Default for TaskArgs {
    fn default() -> Self {
        TaskArgs {
            capacity: None,
            priority: 1,
            resources: Idents::new(),
            schedule: Idents::new(),
            spawn: Idents::new(),
        }
    }
}

impl Parse for TaskArgs {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        parse_args(input, true)
    }
}

// Parser shared by TaskArgs and ExceptionArgs / InterruptArgs
fn parse_args(input: ParseStream<'_>, accept_capacity: bool) -> parse::Result<TaskArgs> {
    if input.is_empty() {
        return Ok(TaskArgs::default());
    }

    let mut capacity = None;
    let mut priority = None;
    let mut resources = None;
    let mut schedule = None;
    let mut spawn = None;

    let content;
    parenthesized!(content in input);
    loop {
        if content.is_empty() {
            break;
        }

        // #ident = ..
        let ident: Ident = content.parse()?;
        let _: Token![=] = content.parse()?;

        let ident_s = ident.to_string();
        match &*ident_s {
            "capacity" if accept_capacity => {
                // #lit
                let lit: LitInt = content.parse()?;

                if lit.suffix() != IntSuffix::None {
                    return Err(parse::Error::new(
                        lit.span(),
                        "this literal must be unsuffixed",
                    ));
                }

                let value = lit.value();
                if value > u64::from(u8::MAX) || value == 0 {
                    return Err(parse::Error::new(
                        lit.span(),
                        "this literal must be in the range 1...255",
                    ));
                }

                capacity = Some(value as u8);
            }
            "priority" => {
                // #lit
                let lit: LitInt = content.parse()?;

                if lit.suffix() != IntSuffix::None {
                    return Err(parse::Error::new(
                        lit.span(),
                        "this literal must be unsuffixed",
                    ));
                }

                let value = lit.value();
                if value > u64::from(u8::MAX) {
                    return Err(parse::Error::new(
                        lit.span(),
                        "this literal must be in the range 0...255",
                    ));
                }

                priority = Some(value as u8);
            }
            "schedule" if cfg!(not(feature = "timer-queue")) => {
                return Err(parse::Error::new(
                    ident.span(),
                    "The `schedule` API requires that the `timer-queue` feature is \
                     enabled in the `cortex-m-rtfm` crate",
                ));
            }
            "resources" | "schedule" | "spawn" => {
                // .. [#(#idents)*]
                let inner;
                bracketed!(inner in content);
                let mut idents = Idents::new();
                for ident in inner.call(Punctuated::<_, Token![,]>::parse_terminated)? {
                    if idents.contains(&ident) {
                        return Err(parse::Error::new(
                            ident.span(),
                            "element appears more than once in list",
                        ));
                    }

                    idents.insert(ident);
                }

                match &*ident_s {
                    "resources" => {
                        if resources.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        resources = Some(idents);
                    }
                    "schedule" => {
                        if schedule.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        schedule = Some(idents);
                    }
                    "spawn" => {
                        if spawn.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        spawn = Some(idents);
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                return Err(parse::Error::new(
                    ident.span(),
                    "expected one of: priority, resources, schedule or spawn",
                ));
            }
        }

        if content.is_empty() {
            break;
        }

        // ,
        let _: Token![,] = content.parse()?;
    }

    Ok(TaskArgs {
        capacity,
        priority: priority.unwrap_or(1),
        resources: resources.unwrap_or(Idents::new()),
        schedule: schedule.unwrap_or(Idents::new()),
        spawn: spawn.unwrap_or(Idents::new()),
    })
}

pub struct Static {
    /// `#[cfg]` attributes
    pub cfgs: Vec<Attribute>,
    /// Attributes that are not `#[cfg]`
    pub attrs: Vec<Attribute>,
    pub ty: Box<Type>,
    pub expr: Box<Expr>,
}

impl Static {
    fn parse(items: Vec<ItemStatic>) -> parse::Result<HashMap<Ident, Static>> {
        let mut statics = HashMap::new();

        for item in items {
            if statics.contains_key(&item.ident) {
                return Err(parse::Error::new(
                    item.ident.span(),
                    "this `static` is listed twice",
                ));
            }

            let (cfgs, attrs) = extract_cfgs(item.attrs);

            statics.insert(
                item.ident,
                Static {
                    cfgs,
                    attrs,
                    ty: item.ty,
                    expr: item.expr,
                },
            );
        }

        Ok(statics)
    }
}

pub struct Task {
    pub args: TaskArgs,
    pub cfgs: Vec<Attribute>,
    pub attrs: Vec<Attribute>,
    pub unsafety: Option<Token![unsafe]>,
    pub inputs: Vec<ArgCaptured>,
    pub statics: HashMap<Ident, Static>,
    pub stmts: Vec<Stmt>,
}

impl Task {
    fn check(args: TaskArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = item.vis == Visibility::Inherited
            && item.constness.is_none()
            && item.asyncness.is_none()
            && item.abi.is_none()
            && item.decl.generics.params.is_empty()
            && item.decl.generics.where_clause.is_none()
            && item.decl.variadic.is_none()
            && is_unit(&item.decl.output);

        let span = item.span();

        if !valid_signature {
            return Err(parse::Error::new(
                span,
                "`task` handlers must have type signature `[unsafe] fn(..)`",
            ));
        }

        let (statics, stmts) = extract_statics(item.block.stmts);

        let mut inputs = vec![];
        for input in item.decl.inputs {
            if let FnArg::Captured(capture) = input {
                inputs.push(capture);
            } else {
                return Err(parse::Error::new(
                    span,
                    "inputs must be named arguments (e.f. `foo: u32`) and not include `self`",
                ));
            }
        }

        match &*item.ident.to_string() {
            "init" | "idle" | "resources" => {
                return Err(parse::Error::new(
                    span,
                    "`task` handlers can NOT be named `idle`, `init` or `resources`",
                ));
            }
            _ => {}
        }

        let (cfgs, attrs) = extract_cfgs(item.attrs);
        Ok(Task {
            args,
            cfgs,
            attrs,
            unsafety: item.unsafety,
            inputs,
            statics: Static::parse(statics)?,
            stmts,
        })
    }
}

pub struct FreeInterrupt {
    pub attrs: Vec<Attribute>,
}

impl FreeInterrupt {
    fn parse(mod_: ItemForeignMod) -> parse::Result<FreeInterrupts> {
        let mut free_interrupts = FreeInterrupts::new();

        for item in mod_.items {
            if let ForeignItem::Fn(f) = item {
                let valid_signature = f.vis == Visibility::Inherited
                    && f.decl.generics.params.is_empty()
                    && f.decl.generics.where_clause.is_none()
                    && f.decl.inputs.is_empty()
                    && f.decl.variadic.is_none()
                    && is_unit(&f.decl.output);

                if !valid_signature {
                    return Err(parse::Error::new(
                        f.span(),
                        "free interrupts must have type signature `fn()`",
                    ));
                }

                if free_interrupts.contains_key(&f.ident) {
                    return Err(parse::Error::new(
                        f.ident.span(),
                        "this interrupt appears twice",
                    ));
                }

                free_interrupts.insert(f.ident, FreeInterrupt { attrs: f.attrs });
            } else {
                return Err(parse::Error::new(
                    mod_.abi.extern_token.span(),
                    "`extern` block should only contains functions",
                ));
            }
        }

        Ok(free_interrupts)
    }
}

fn eq(attr: &Attribute, name: &str) -> bool {
    attr.style == AttrStyle::Outer && attr.path.segments.len() == 1 && {
        let pair = attr.path.segments.first().unwrap();
        let segment = pair.value();
        segment.arguments == PathArguments::None && segment.ident.to_string() == name
    }
}

fn extract_cfgs(attrs: Vec<Attribute>) -> (Vec<Attribute>, Vec<Attribute>) {
    let mut cfgs = vec![];
    let mut not_cfgs = vec![];

    for attr in attrs {
        if eq(&attr, "cfg") {
            cfgs.push(attr);
        } else {
            not_cfgs.push(attr);
        }
    }

    (cfgs, not_cfgs)
}

/// Extracts `static mut` vars from the beginning of the given statements
fn extract_statics(stmts: Vec<Stmt>) -> (Statics, Vec<Stmt>) {
    let mut istmts = stmts.into_iter();

    let mut statics = Statics::new();
    let mut stmts = vec![];
    while let Some(stmt) = istmts.next() {
        match stmt {
            Stmt::Item(Item::Static(var)) => {
                if var.mutability.is_some() {
                    statics.push(var);
                } else {
                    stmts.push(Stmt::Item(Item::Static(var)));
                    break;
                }
            }
            _ => {
                stmts.push(stmt);
                break;
            }
        }
    }

    stmts.extend(istmts);

    (statics, stmts)
}

fn extract_assignments(stmts: Vec<Stmt>) -> (Vec<Stmt>, Vec<Assign>) {
    let mut istmts = stmts.into_iter().rev();

    let mut assigns = vec![];
    let mut stmts = vec![];
    while let Some(stmt) = istmts.next() {
        match stmt {
            Stmt::Semi(Expr::Assign(assign), semi) => {
                if let Expr::Path(ref expr) = *assign.left {
                    if expr.path.segments.len() == 1 {
                        assigns.push(Assign {
                            attrs: assign.attrs,
                            left: expr.path.segments[0].ident.clone(),
                            right: assign.right,
                        });
                        continue;
                    }
                }

                stmts.push(Stmt::Semi(Expr::Assign(assign), semi));
            }
            _ => {
                stmts.push(stmt);
                break;
            }
        }
    }

    stmts.extend(istmts);

    (stmts.into_iter().rev().collect(), assigns)
}

fn is_bottom(ty: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = ty {
        if let Type::Never(_) = **ty {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn is_unit(ty: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = ty {
        if let Type::Tuple(ref tuple) = **ty {
            tuple.elems.is_empty()
        } else {
            false
        }
    } else {
        true
    }
}
