use std::{env, fs, process};

use veonep::interpreter::Interpreter;
use veonep::parser::Parser;
use veonep::scanner::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: veonep <file>");
        process::exit(64);
    }

    let source = fs::read_to_string(&args[1]).unwrap_or_else(|err| {
        eprintln!("Failed to read {}: {err}", args[1]);
        process::exit(65);
    });

    let mut scanner = Scanner::new(source);
    let tokens = match scanner.tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("{err}");
            process::exit(65);
        }
    };

    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(stmts) => stmts,
        Err(err) => {
            eprintln!("{err}");
            process::exit(65);
        }
    };

    let mut interpreter = Interpreter::new();
    match interpreter.interpret(&statements) {
        Ok(Some(value)) => println!("{value}"),
        Ok(None) => (),
        Err(err) => {
            eprintln!("{err}");
            process::exit(70);
        }
    }
}
