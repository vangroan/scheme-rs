use std::io::{self, Write};
use std::{env, fs};

use scheme_engine::{self, Expr};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(file_path) => run_file(file_path),
        None => run_repl(),
    }
}

fn run_file(file_path: &str) {
    match fs::read_to_string(file_path) {
        Ok(script) => {
            // Global environment
            let env = scheme_engine::new_env().expect("failed creating new core environment");

            let expr =
                scheme_engine::parse(script.as_str(), true).expect("failed to parse program");

            let program =
                scheme_engine::compile(env.clone(), &expr).expect("failed to compile program");

            let _value0 = scheme_engine::eval(program.closure().clone()).expect("runtime error");
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
                    Ok(program) => {
                        println!("bytecode:");
                        for (index, op) in program
                            .closure()
                            .borrow()
                            .procedure()
                            .bytecode()
                            .iter()
                            .enumerate()
                        {
                            println!("  {index:>6} : {op:?}");
                        }

                        // Run closure in VM
                        match scheme_engine::eval(program.closure().clone()) {
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
