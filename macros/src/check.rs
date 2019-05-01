use std::{collections::HashSet, iter};

use proc_macro2::Span;
use syn::{parse, spanned::Spanned, Block, Expr, Stmt};

use crate::syntax::App;

pub fn app(app: &App) -> parse::Result<()> {
    // Check that all referenced resources have been declared
    for res in app
        .idle
        .as_ref()
        .map(|idle| -> Box<dyn Iterator<Item = _>> { Box::new(idle.args.resources.iter()) })
        .unwrap_or_else(|| Box::new(iter::empty()))
        .chain(&app.init.args.resources)
        .chain(app.exceptions.values().flat_map(|e| &e.args.resources))
        .chain(app.interrupts.values().flat_map(|i| &i.args.resources))
        .chain(app.tasks.values().flat_map(|t| &t.args.resources))
    {
        if !app.resources.contains_key(res) {
            return Err(parse::Error::new(
                res.span(),
                "this resource has NOT been declared",
            ));
        }
    }

    // Check that late resources have not been assigned to `init`
    for res in &app.init.args.resources {
        if app.resources.get(res).unwrap().expr.is_none() {
            return Err(parse::Error::new(
                res.span(),
                "late resources can NOT be assigned to `init`",
            ));
        }
    }

    // Check that `init` returns `LateResources` if there's any declared late resource
    if !app.init.returns_late_resources && app.resources.iter().any(|(_, res)| res.expr.is_none()) {
        return Err(parse::Error::new(
            app.init.span,
            "late resources have been specified so `init` must return `init::LateResources`",
        ));
    }

    // Check that all referenced tasks have been declared
    for task in app
        .idle
        .as_ref()
        .map(|idle| -> Box<dyn Iterator<Item = _>> {
            Box::new(idle.args.schedule.iter().chain(&idle.args.spawn))
        })
        .unwrap_or_else(|| Box::new(iter::empty()))
        .chain(&app.init.args.schedule)
        .chain(&app.init.args.spawn)
        .chain(
            app.exceptions
                .values()
                .flat_map(|e| e.args.schedule.iter().chain(&e.args.spawn)),
        )
        .chain(
            app.interrupts
                .values()
                .flat_map(|i| i.args.schedule.iter().chain(&i.args.spawn)),
        )
        .chain(
            app.tasks
                .values()
                .flat_map(|t| t.args.schedule.iter().chain(&t.args.spawn)),
        )
    {
        if !app.tasks.contains_key(task) {
            return Err(parse::Error::new(
                task.span(),
                "this task has NOT been declared",
            ));
        }
    }

    // Check that there are enough free interrupts to dispatch all tasks
    let ndispatchers = app
        .tasks
        .values()
        .map(|t| t.args.priority)
        .collect::<HashSet<_>>()
        .len();
    if ndispatchers > app.free_interrupts.len() {
        return Err(parse::Error::new(
            Span::call_site(),
            &*format!(
                "{} free interrupt{} (`extern {{ .. }}`) {} required to dispatch all soft tasks",
                ndispatchers,
                if ndispatchers > 1 { "s" } else { "" },
                if ndispatchers > 1 { "are" } else { "is" },
            ),
        ));
    }

    // Check that free interrupts are not being used
    for (handler, interrupt) in &app.interrupts {
        let name = interrupt.args.binds(handler);

        if app.free_interrupts.contains_key(name) {
            return Err(parse::Error::new(
                name.span(),
                "free interrupts (`extern { .. }`) can't be used as interrupt handlers",
            ));
        }
    }

    // Check that `init` contains no early returns *if* late resources exist and `init` signature is
    // `fn()`
    if app.resources.values().any(|res| res.expr.is_none()) {
        if !app.init.returns_late_resources {
            for stmt in &app.init.stmts {
                noreturn_stmt(stmt)?;
            }
        }
    } else if app.init.returns_late_resources {
        return Err(parse::Error::new(
            Span::call_site(),
            "`init` signature must be `fn(init::Context)` if there are no late resources",
        ));
    }

    Ok(())
}

// checks that the given block contains no instance of `return`
fn noreturn_block(block: &Block) -> Result<(), parse::Error> {
    for stmt in &block.stmts {
        noreturn_stmt(stmt)?;
    }

    Ok(())
}

// checks that the given statement contains no instance of `return`
fn noreturn_stmt(stmt: &Stmt) -> Result<(), parse::Error> {
    match stmt {
        // `let x = ..` -- this may contain a return in the RHS
        Stmt::Local(local) => {
            if let Some(ref init) = local.init {
                noreturn_expr(&init.1)?
            }
        }

        // items have no effect on control flow
        Stmt::Item(..) => {}

        Stmt::Expr(expr) => noreturn_expr(expr)?,

        Stmt::Semi(expr, ..) => noreturn_expr(expr)?,
    }

    Ok(())
}

// checks that the given expression contains no `return`
fn noreturn_expr(expr: &Expr) -> Result<(), parse::Error> {
    match expr {
        Expr::Box(b) => noreturn_expr(&b.expr)?,

        Expr::InPlace(ip) => {
            noreturn_expr(&ip.place)?;
            noreturn_expr(&ip.value)?;
        }

        Expr::Array(a) => {
            for elem in &a.elems {
                noreturn_expr(elem)?;
            }
        }

        Expr::Call(c) => {
            noreturn_expr(&c.func)?;

            for arg in &c.args {
                noreturn_expr(arg)?;
            }
        }

        Expr::MethodCall(mc) => {
            noreturn_expr(&mc.receiver)?;

            for arg in &mc.args {
                noreturn_expr(arg)?;
            }
        }

        Expr::Tuple(t) => {
            for elem in &t.elems {
                noreturn_expr(elem)?;
            }
        }

        Expr::Binary(b) => {
            noreturn_expr(&b.left)?;
            noreturn_expr(&b.right)?;
        }

        Expr::Unary(u) => {
            noreturn_expr(&u.expr)?;
        }

        Expr::Lit(..) => {}

        Expr::Cast(c) => {
            noreturn_expr(&c.expr)?;
        }

        Expr::Type(t) => {
            noreturn_expr(&t.expr)?;
        }

        Expr::Let(l) => {
            noreturn_expr(&l.expr)?;
        }

        Expr::If(i) => {
            noreturn_expr(&i.cond)?;

            noreturn_block(&i.then_branch)?;

            if let Some(ref e) = i.else_branch {
                noreturn_expr(&e.1)?;
            }
        }

        Expr::While(w) => {
            noreturn_expr(&w.cond)?;
            noreturn_block(&w.body)?;
        }

        Expr::ForLoop(fl) => {
            noreturn_expr(&fl.expr)?;
            noreturn_block(&fl.body)?;
        }

        Expr::Loop(l) => {
            noreturn_block(&l.body)?;
        }

        Expr::Match(m) => {
            noreturn_expr(&m.expr)?;

            for arm in &m.arms {
                if let Some(g) = &arm.guard {
                    noreturn_expr(&g.1)?;
                }

                noreturn_expr(&arm.body)?;
            }
        }

        // we don't care about `return`s inside closures
        Expr::Closure(..) => {}

        Expr::Unsafe(u) => {
            noreturn_block(&u.block)?;
        }

        Expr::Block(b) => {
            noreturn_block(&b.block)?;
        }

        Expr::Assign(a) => {
            noreturn_expr(&a.left)?;
            noreturn_expr(&a.right)?;
        }

        Expr::AssignOp(ao) => {
            noreturn_expr(&ao.left)?;
            noreturn_expr(&ao.right)?;
        }

        Expr::Field(f) => {
            noreturn_expr(&f.base)?;
        }

        Expr::Index(i) => {
            noreturn_expr(&i.expr)?;
            noreturn_expr(&i.index)?;
        }

        Expr::Range(r) => {
            if let Some(ref f) = r.from {
                noreturn_expr(f)?;
            }

            if let Some(ref t) = r.to {
                noreturn_expr(t)?;
            }
        }

        Expr::Path(..) => {}

        Expr::Reference(r) => {
            noreturn_expr(&r.expr)?;
        }

        Expr::Break(b) => {
            if let Some(ref e) = b.expr {
                noreturn_expr(e)?;
            }
        }

        Expr::Continue(..) => {}

        Expr::Return(r) => {
            return Err(parse::Error::new(
                r.span(),
                "`init` is *not* allowed to early return",
            ));
        }

        // we can not analyze this
        Expr::Macro(..) => {}

        Expr::Struct(s) => {
            for field in &s.fields {
                noreturn_expr(&field.expr)?;
            }

            if let Some(ref rest) = s.rest {
                noreturn_expr(rest)?;
            }
        }

        Expr::Repeat(r) => {
            noreturn_expr(&r.expr)?;
            noreturn_expr(&r.len)?;
        }

        Expr::Paren(p) => {
            noreturn_expr(&p.expr)?;
        }

        Expr::Group(g) => {
            noreturn_expr(&g.expr)?;
        }

        Expr::Try(t) => {
            noreturn_expr(&t.expr)?;
        }

        // we don't care about `return`s inside async blocks
        Expr::Async(..) => {}

        Expr::TryBlock(tb) => {
            noreturn_block(&tb.block)?;
        }

        Expr::Yield(y) => {
            if let Some(expr) = &y.expr {
                noreturn_expr(expr)?;
            }
        }

        // we can not analyze this
        Expr::Verbatim(..) => {}
    }

    Ok(())
}
