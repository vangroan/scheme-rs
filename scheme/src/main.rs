use std::io::{self, Write};
use std::{env, fs};

use clap::parser;
use scheme_engine::{self, Expr};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(file_path) => run_file(file_path),
        None => run_repl_v2(),
    }
}

/// The REPL evaluate the following in any order:
///
/// - Import statements
/// - Definitions
/// - Expressions
fn run_repl_v2() {
    let mut buf = String::new();
    let stdin = io::stdin();
    let mut count = 0;

    println!("Scheme REPL v2.0");

    let store = scheme_core::Store::new();

    loop {
        count += 1;
        buf.clear();
        print!("{count} > ");
        let _ = io::stdout().flush();

        stdin.read_line(&mut buf).expect("read stdin");
        let code = &buf.as_str();

        dump_tokens(code);
        println!("---");
        dump_value(&store, code);
    }
}

fn dump_tokens(code: &str) {
    for token in scheme_core::lexer::Lexer::new(code) {
        println!("{token:?}");
    }
}

fn dump_value(store: &scheme_core::Store, code: &str) {
    let lexer = scheme_core::lexer::Lexer::new(code);
    let token_stream = scheme_core::lexer::TokenStream::new(lexer);
    let mut parser = scheme_core::parser::Parser::new(&store, token_stream);

    match parser.parse_expression() {
        Ok(expr) => {
            let repr = scheme_core::display::ReprDisplay::new(&expr);
            println!("parse:\n\t{}", repr);
        }
        Err(err) => {
            eprintln!("error: {err}");
        }
    }
}

fn run_file(file_path: &str) {
    match fs::read_to_string(file_path) {
        Ok(script) => {
            // Global environment
            let env = scheme_engine::new_env().expect("failed creating new core environment");

            let expr =
                scheme_engine::parse(script.as_str(), true).expect("failed to parse program");

            let closure =
                scheme_engine::compile(env.clone(), &expr).expect("failed to compile program");

            let _value0 = scheme_engine::eval(closure).expect("runtime error");
        }
        Err(err) => {
            eprintln!("failed to open file: {err}");
        }
    }
}

fn run_repl() {
    let mut buf = String::new();
    let stdin = io::stdin();
    let mut count = 0;

    // Console environment.
    let env = scheme_engine::new_env().expect("failed creating new core environment");

    loop {
        count += 1;
        buf.clear();
        print!("{count} > ");
        let _ = io::stdout().flush();
        stdin.read_line(&mut buf).expect("read stdin");

        match scheme_engine::parse(buf.as_str(), true) {
            Ok(expr) => {
                println!("parse:\n\t{:#?}", expr);

                match scheme_engine::compile(env.clone(), &expr) {
                    Ok(closure) => {
                        println!("bytecode:");
                        for (index, op) in
                            closure.borrow().procedure().bytecode().iter().enumerate()
                        {
                            println!("  {index:>6} : {op:?}");
                        }

                        // Run closure in VM
                        match scheme_engine::eval(closure) {
                            Ok(Expr::Void) => {
                                // Don't print a #!void, it's the "nothing" value
                            }
                            Ok(value) => {
                                println!("{}", value.repr());
                            }
                            Err(err) => {
                                eprintln!("error: {err}");
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("error: {err}");
                    }
                }
            }
            Err(err) => {
                eprintln!("error: {err}");
            }
        }
    }
}
