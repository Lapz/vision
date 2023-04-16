use ast::prelude::ItemKind;
use errors::Level;
use syntax::Parser;

use crate::Resolver;

struct ExpectedDiagnostic {
    level: Level,
    msg: &'static str,
}

macro_rules! setup_reporter {
    ($file:expr) => {{
        let file = $file;

        let parser = Parser::new(file);

        let (program, symbols) = parser.parse().unwrap();

        let mut resolver = Resolver::new(symbols);

        let errors = resolver.resolve_program(&program);

        (errors, resolver)
    }};
}

macro_rules! assert_diagnostics {
    ($expected:expr,$reporter:ident) => {{
        let expected = $expected;

        let mut found = 0;

        for diagnostic in $reporter.diagnostics().iter() {
            for exp in expected.iter() {
                if diagnostic.level == exp.level && diagnostic.msg == exp.msg {
                    found += 1;
                }
            }
        }

        assert_eq!(found, expected.len())
    }};
}

#[test]
fn it_works() {
    let (reporter, _) = setup_reporter!(
        "fn main() {
                let a := 10;
                let b := 10;


                return a+b;
            }"
    );

    assert!(!reporter.has_error())
}

#[test]
fn it_has_different_environments_for_types() {
    let (_, mut resolver) = setup_reporter!(
        "type a = number;
                fn main() {
                    let a:a := 10;
                    let b:a := 20;
                    return a+b;
                }"
    );

    let a = resolver.symbols.intern("a");

    assert!(resolver.items.get(&(a, ItemKind::Type)).is_some());
    assert!(resolver.items.get(&(a, ItemKind::Type)).is_some())
}

#[test]
fn it_resolves_functions_and_types() {
    let (_, mut resolver) = setup_reporter!(
        "type a = number;
                fn main() {
                    let a:a := 10;
                    let b:a := 20;
                    return a+b;
                }"
    );

    let a = resolver.symbols.intern("a");

    let main = resolver.symbols.intern("main");

    assert!(resolver.items.get(&(a, ItemKind::Type)).is_some());
    assert!(resolver.items.get(&(main, ItemKind::Value)).is_some())
}

#[test]
fn it_errors_on_unknown_identifier() {
    let (reporter, _) = setup_reporter!(
        "
                fn main() {
                    let a := 10;
                    let c := 20;

                    return a+b;
                }"
    );

    let expected = [ExpectedDiagnostic {
        level: Level::Error,
        msg: "Unknown identifier `b`",
    }];

    let mut found = 0;

    for diagnostic in reporter.diagnostics().iter() {
        println!("{:?}", diagnostic.msg);
        for exp in expected.iter() {
            if diagnostic.level == exp.level && diagnostic.msg == exp.msg {
                found += 1;
            }
        }
    }

    assert_eq!(found, expected.len())
}

#[test]
fn it_warns_on_var_shadowing() {
    let (reporter, _) = setup_reporter!(
        "
                fn main() {
                    let a := 10;
                    {
                        let a := 20;
                    }

                    return a;
                }"
    );

    assert_diagnostics!(
        [ExpectedDiagnostic {
            level: Level::Warn,
            msg: "The identifier `a` has already been declared.",
        }],
        reporter
    )
}

#[test]
fn it_warns_on_unused_variables() {
    let (reporter, _) = setup_reporter!(
        "
                 fn main() {
                    let a := 10;
                    let b := 10;

                    return a;
                }"
    );

    assert_diagnostics!(
        [ExpectedDiagnostic {
            level: Level::Warn,
            msg: "Unused variable `b`",
        },],
        reporter
    )
}

#[test]
fn it_does_not_warn_on_main() {
    let (reporter, _) = setup_reporter!("fn main() {}");

    assert!(!reporter.has_error())
}
