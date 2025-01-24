use crate::join_with;
use glob::glob;
use std::collections::BTreeMap;

use std::path::PathBuf;

#[test]
fn snapshot_tests() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    for i in glob(&(d.to_str().unwrap().to_string() + "/**/*")).unwrap() {
        if let Ok(file) = i
            && let Ok(contents) = std::fs::read_to_string(file.clone())
        {
            let book = crate::syntax::parser::parse_file(&contents);

            match book {
                Ok(book) => {
                    let mut compiler = crate::syntax::compiler::Compiler::default();
                    compiler.compile_book(book);
                    let mut main_net = compiler.main_net();

                    // Ensure the main net is compiled correctly
                    let mut scope = std::collections::BTreeMap::new();
                    let show_agent = |x| format!("{:?}", x);
                    insta::assert_snapshot!(
                        format!("{}/compilation", file.display()),
                        main_net.show_net(&show_agent, &mut scope, 0)
                    );

                    // Ensure the main net is normalized correctly
                    main_net.normal(crate::net::rules::apply_rule);
                    let mut scope = std::collections::BTreeMap::new();
                    let show_agent = |x| format!("{:?}", x);
                    insta::assert_snapshot!(
                        format!("{}/normalization", file.display()),
                        main_net.show_net(&show_agent, &mut scope, 0)
                    );

                    let mut ctx = BTreeMap::new();

                    let trees = main_net.substitute_iter(main_net.ports.iter());
                    let types = crate::types::infer(trees);

                    let result = format!(
                        "|- {}",
                        join_with(
                            types.into_iter().map(|x| x.show(&mut ctx)),
                            ", ".to_string()
                        )
                    );
                    insta::assert_snapshot!(format!("{}/typing", file.display()), result);
                }
                Err(e) => {
                    insta::assert_snapshot!(format!("{}/compilation", file.display()), e);
                }
            }
        }
    }
}
