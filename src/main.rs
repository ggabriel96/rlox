use std::io::{BufRead, Write};

use clap::{AppSettings, Clap};
use rlox::interpreter::interpret;
use rlox::lexing::Scanner;
use rlox::parsing::parse;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Path of script to run
    file: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.file {
        Some(path) => run_file(path),
        None => run_prompt(),
    }
}

fn run(line: String) {
    let mut scanner = Scanner::new(line);
    match scanner.scan() {
        Ok(tokens) => {
            // for tok in tokens.iter() {
            //     println!("{:?}", tok);
            // }
            if tokens.len() > 1 {
                match parse(&tokens) {
                    Ok(statements) => {
                        // println!("{:?}", statements);
                        match interpret(statements) {
                            Ok(()) => (),
                            Err(runtime_error) => eprintln!("{:?}", runtime_error),
                        }
                    }
                    Err(parse_error) => eprintln!("{:?}", parse_error),
                }
            }
        }
        Err(lexing_error) => eprintln!("{:?}", lexing_error),
    }
}

fn run_file(path: String) {
    let content = std::fs::read_to_string(path).unwrap();
    run(content);
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    loop {
        print!("> ");
        stdout.lock().flush().unwrap();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => (),
            Err(error) => panic!("{}", error),
        }
        run(line);
    }
}
