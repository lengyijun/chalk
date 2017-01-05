use chalk_rust_parse;
use errors::*;
use ir;
use lower::*;
use solve::goal::Prove;
use solve::solver::Solver;
use std::sync::Arc;

fn parse_and_lower_program(text: &str) -> Result<ir::Program> {
    chalk_rust_parse::parse_program(text)?.lower()
}

fn parse_and_lower_goal(program: &ir::Program, text: &str) -> Result<Box<ir::Goal>> {
    chalk_rust_parse::parse_goal(text)?.lower(program)
}

macro_rules! test {
    (test $name:ident { program $program:tt $(goal $goal:tt yields { $expected:expr })* }) => {
        #[test]
        fn $name() {
            solve_goal(stringify!($program), vec![$((stringify!($goal), $expected)),*])
        }
    }
}

fn solve_goal(program_text: &str,
              goals: Vec<(&str, &str)>)
{
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));
    let program = Arc::new(parse_and_lower_program(&program_text[1..program_text.len()-1]).unwrap());
    ir::set_current_program(&program, || {
        for (goal_text, expected) in goals {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = parse_and_lower_goal(&program, &goal_text[1..goal_text.len()-1]).unwrap();
            let result = match Prove::new(&mut Solver::new(&program), goal).solve() {
                Ok(v) => format!("{:#?}", v),
                Err(e) => format!("{}", e),
            };
            println!("expected:\n{:?}", expected);
            println!("actual:\n{:#?}", result);

            // remove all whitespace:
            let expected1: String = expected.chars().filter(|w| !w.is_whitespace()).collect();
            let result1: String = result.chars().filter(|w| !w.is_whitespace()).collect();
            assert_eq!(expected1, result1);
        }
    });
}

test! {
    test prove_clone {
        program {
            struct Foo { }
            struct Bar { }
            struct Vec<T> { }
            trait Clone { }
            impl<T> Clone for Vec<T> where T: Clone { }
            impl Clone for Foo { }
        }

        goal {
            Vec<Foo>: Clone
        } yields {
            "Yes"
        }

        goal {
            Foo: Clone
        } yields {
            "Yes"
        }

        goal {
            Bar: Clone
        } yields {
            "`Bar as Clone` is not implemented in environment \
             `Environment { universe: UniverseIndex { counter: 0 }, clauses: [] }`"
        }

        goal {
            Vec<Bar>: Clone
        } yields {
            "`Vec<Bar> as Clone` is not implemented in environment \
             `Environment { universe: UniverseIndex { counter: 0 }, clauses: [] }`"
        }
    }
}
