use std::collections::{HashMap, HashSet};

use syn::{self, DelimToken, Ident, IntTy, Lit, Token, TokenTree};

use syntax::{App, Idle, Init, Kind, Resource, Statics, Task, Tasks};

pub fn app(input: &str) -> App {
    let tts = syn::parse_token_trees(input).unwrap();

    let mut device = None;
    let mut init = None;
    let mut idle = None;
    let mut resources = None;
    let mut tasks = None;

    let mut tts = tts.into_iter();
    while let Some(tt) = tts.next() {
        let id = if let TokenTree::Token(Token::Ident(id)) = tt {
            id
        } else {
            panic!("expected ident, found {:?}", tt);
        };

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Colon)),
            "expected colon, found {:?}",
            tt
        );

        match id.as_ref() {
            "device" => {
                assert!(device.is_none(), "duplicated device field");

                let mut pieces = vec![];

                loop {
                    if let Some(tt) = tts.next() {
                        if tt == TokenTree::Token(Token::Comma) {
                            break;
                        } else {
                            pieces.push(tt);
                        }
                    } else {
                        panic!("expected path, found EOM");
                    }
                }

                device = Some(quote!(#(#pieces)*));
                continue;
            }
            "idle" => {
                assert!(idle.is_none(), "duplicated idle field");

                let tt = tts.next();
                if let Some(TokenTree::Delimited(block)) = tt {
                    assert_eq!(
                        block.delim,
                        DelimToken::Brace,
                        "expected brace, found {:?}",
                        block.delim
                    );

                    idle = Some(super::parse::idle(block.tts));
                } else {
                    panic!("expected block, found {:?}", tt);
                }
            }
            "init" => {
                assert!(init.is_none(), "duplicated init field");

                let tt = tts.next();
                if let Some(TokenTree::Delimited(block)) = tt {
                    assert_eq!(
                        block.delim,
                        DelimToken::Brace,
                        "expected brace, found {:?}",
                        block.delim
                    );

                    init = Some(super::parse::init(block.tts));
                } else {
                    panic!("expected block, found {:?}", tt);
                }
            }
            "resources" => {
                assert!(resources.is_none(), "duplicated resources field");

                let tt = tts.next();
                if let Some(TokenTree::Delimited(block)) = tt {
                    assert_eq!(
                        block.delim,
                        DelimToken::Brace,
                        "expected brace, found {:?}",
                        block.delim
                    );

                    resources = Some(super::parse::statics(block.tts));
                }
            }
            "tasks" => {
                assert!(tasks.is_none(), "duplicated tasks field");

                let tt = tts.next();
                if let Some(TokenTree::Delimited(block)) = tt {
                    assert_eq!(
                        block.delim,
                        DelimToken::Brace,
                        "expected brace, found {:?}",
                        block.delim
                    );

                    tasks = Some(super::parse::tasks(block.tts));
                }
            }
            id => panic!("unexpected field {}", id),
        }

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Comma)),
            "expected comma, found {:?}",
            tt
        );
    }

    App {
        device: device.expect("device field is missing"),
        idle: idle.expect("idle field is missing"),
        init: init.expect("init field is missing"),
        resources: resources.unwrap_or(HashMap::new()),
        tasks: tasks.unwrap_or(HashMap::new()),
    }
}

pub fn idle(tts: Vec<TokenTree>) -> Idle {
    let mut tts = tts.into_iter();

    let mut local = None;
    let mut path = None;
    let mut resources = None;
    while let Some(tt) = tts.next() {
        let id = if let TokenTree::Token(Token::Ident(id)) = tt {
            id
        } else {
            panic!("expected ident, found {:?}", tt);
        };

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Colon)),
            "expected colon, found {:?}",
            tt
        );

        match id.as_ref() {
            "local" => {
                assert!(local.is_none(), "duplicated local field");

                let tt = tts.next();
                if let Some(TokenTree::Delimited(block)) = tt {
                    assert_eq!(
                        block.delim,
                        DelimToken::Brace,
                        "expected brace, found {:?}",
                        block.delim
                    );

                    local = Some(super::parse::statics(block.tts));
                } else {
                    panic!("expected block, found {:?}", tt);
                }
            }
            "path" => {
                assert!(path.is_none(), "duplicated path field");

                let mut pieces = vec![];
                loop {
                    let tt = tts.next()
                        .expect("expected comma, found end of macro");

                    if tt == TokenTree::Token(Token::Comma) {
                        path = Some(quote!(#(#pieces)*));
                        break;
                    } else {
                        pieces.push(tt);
                    }
                }

                continue;
            }
            "resources" => {
                assert!(resources.is_none(), "duplicated resources field");

                let tt = tts.next();
                if let Some(TokenTree::Delimited(array)) = tt {
                    assert_eq!(
                        array.delim,
                        DelimToken::Bracket,
                        "expected bracket, found {:?}",
                        array.delim
                    );

                    resources = Some(super::parse::idents(array.tts));

                } else {
                    panic!("expected array, found {:?}", tt);
                }
            }
            id => panic!("unexpected field {}", id),
        }

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Comma)),
            "expected comma, found {:?}",
            tt
        );
    }

    Idle {
        local: local.unwrap_or(HashMap::new()),
        path: path.expect("path field is missing"),
        resources: resources.unwrap_or(HashSet::new()),
    }
}

pub fn init(tts: Vec<TokenTree>) -> Init {
    let mut tts = tts.into_iter();

    let mut path = None;
    while let Some(tt) = tts.next() {
        let id = if let TokenTree::Token(Token::Ident(id)) = tt {
            id
        } else {
            panic!("expected ident, found {:?}", tt);
        };

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Colon)),
            "expected colon, found {:?}",
            tt
        );

        match id.as_ref() {
            "path" => {
                let mut pieces = vec![];
                loop {
                    let tt = tts.next()
                        .expect("expected comma, found end of macro");

                    if tt == TokenTree::Token(Token::Comma) {
                        path = Some(quote!(#(#pieces)*));
                        break;
                    } else {
                        pieces.push(tt);
                    }
                }
            }
            id => panic!("unexpected field {}", id),
        }
    }

    Init { path: path.expect("path field is missing") }
}

fn idents(tts: Vec<TokenTree>) -> HashSet<Ident> {
    let mut idents = HashSet::new();

    let mut tts = tts.into_iter().peekable();
    while let Some(tt) = tts.next() {
        if let TokenTree::Token(Token::Ident(id)) = tt {
            assert!(!idents.contains(&id), "ident {} already listed", id);
            idents.insert(id);

            if let Some(tt) = tts.next() {
                assert_eq!(tt, TokenTree::Token(Token::Comma));

                if tts.peek().is_none() {
                    break;
                }
            } else {
                break;
            }
        } else {
            panic!("expected ident, found {:?}", tt);
        };
    }

    idents
}

pub fn statics(tts: Vec<TokenTree>) -> Statics {
    let mut resources = HashMap::new();

    let mut tts = tts.into_iter();
    while let Some(tt) = tts.next() {
        let name = if let TokenTree::Token(Token::Ident(ident)) = tt {
            ident
        } else {
            panic!("expected ident, found {:?}", tt);
        };

        assert!(
            !resources.contains_key(&name),
            "resource {} already listed",
            name
        );

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Colon)),
            "expected comma, found {:?}",
            tt
        );

        let mut pieces = vec![];
        loop {
            if let Some(tt) = tts.next() {
                if tt == TokenTree::Token(Token::Eq) {
                    break;
                } else {
                    pieces.push(tt);
                }
            } else {
                panic!("expected type, found EOM");
            }
        }

        let ty = quote!(#(#pieces)*);

        let mut pieces = vec![];
        loop {
            if let Some(tt) = tts.next() {
                if tt == TokenTree::Token(Token::Semi) {
                    break;
                } else {
                    pieces.push(tt);
                }
            } else {
                panic!("expected expression, found EOM");
            }
        }

        let expr = quote!(#(#pieces)*);

        let resource = Resource { expr, ty };
        resources.insert(name, resource);
    }

    resources
}

pub fn tasks(tts: Vec<TokenTree>) -> Tasks {
    let mut tasks = HashMap::new();

    let mut tts = tts.into_iter();
    while let Some(tt) = tts.next() {
        let name = if let TokenTree::Token(Token::Ident(ident)) = tt {
            ident
        } else {
            panic!("expected ident, found {:?}", tt);
        };

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Colon)),
            "expected colon, found {:?}",
            tt
        );

        let tt = tts.next();
        if let Some(TokenTree::Delimited(block)) = tt {
            assert_eq!(
                block.delim,
                DelimToken::Brace,
                "expected brace, found {:?}",
                block.delim
            );

            assert!(!tasks.contains_key(&name), "task {} already listed", name);
            tasks.insert(name, super::parse::task(block.tts));
        } else {
            panic!("expected block, found {:?}", tt);
        }

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Comma)),
            "expected comma, found {:?}",
            tt
        );
    }

    tasks
}

/// Parses the body of a task
///
/// ```
///     enabled: true,
///     priority: 1,
///     resources: [R1, TIM2],
/// ```
///
/// the `enabled` field is optional and distinguishes interrupts from
/// exceptions. Interrupts have an `enabled` field, whereas exceptions don't.
fn task(tts: Vec<TokenTree>) -> Task {
    let mut enabled = None;
    let mut priority = None;
    let mut resources = None;

    let mut tts = tts.into_iter();
    while let Some(tt) = tts.next() {
        let ident = if let TokenTree::Token(Token::Ident(ident)) = tt {
            ident
        } else {
            panic!("expected ident, found {:?}", tt);
        };

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Colon)),
            "expected colon, found {:?}",
            tt
        );

        match ident.as_ref() {
            "enabled" => {
                assert!(enabled.is_none(), "duplicated enabled field");

                let tt = tts.next();

                if let Some(TokenTree::Token(Token::Literal(lit))) = tt {
                    if let Lit::Bool(b) = lit {
                        enabled = Some(b);
                    } else {
                        panic!("`enabled` value must be a boolean");
                    }
                } else {
                    panic!("expected literal, found {:?}", tt);
                }
            }
            "priority" => {
                assert!(priority.is_none(), "duplicated priority field");

                let tt = tts.next();

                if let Some(TokenTree::Token(Token::Literal(lit))) = tt {
                    if let Lit::Int(val, ty) = lit {
                        assert_eq!(
                            ty,
                            IntTy::Unsuffixed,
                            "`priority` value must be an unsuffixed value"
                        );

                        assert!(
                            val < 256,
                            "`priority` value must be less than 256"
                        );

                        priority = Some(val as u8);
                    } else {
                        panic!("enabled value must be a boolean");
                    }
                } else {
                    panic!("expected literal, found {:?}", tt);
                }
            }
            "resources" => {
                assert!(resources.is_none(), "duplicated resources field");

                let tt = tts.next();
                if let Some(TokenTree::Delimited(block)) = tt {
                    assert_eq!(
                        block.delim,
                        DelimToken::Bracket,
                        "expected bracket, found {:?}",
                        block.delim
                    );

                    resources = Some(super::parse::idents(block.tts));
                } else {
                    panic!("expected block, found {:?}", tt);
                }
            }
            id => panic!("unexpected field {}", id),
        }

        let tt = tts.next();
        assert_eq!(
            tt,
            Some(TokenTree::Token(Token::Comma)),
            "expected comma, found {:?}",
            tt
        );
    }

    let resources = resources.unwrap_or(HashSet::new());
    let priority = priority.expect("priority field is missing");
    let kind = if let Some(enabled) = enabled {
        Kind::Interrupt { enabled }
    } else {
        Kind::Exception
    };

    Task {
        kind,
        priority,
        resources,
    }
}
