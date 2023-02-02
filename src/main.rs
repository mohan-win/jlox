use jlox::error::error_at_runtime;
use jlox::interpreter::Interpreter;
use jlox::parser::Parser;
use jlox::resolver::Resolver;
use jlox::scanner::Scanner;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: jlox [script]");
        ExitCode::from(ExitCode::FAILURE)
    } else if args.len() == 2 {
        match run_file(&args[1]) {
            Err(err) => {
                eprintln!("Erred out {:?}", err);
                ExitCode::FAILURE
            }
            Ok(()) => ExitCode::SUCCESS,
        }
    } else {
        match run_prompt() {
            Err(err) => {
                eprintln!("Erred out {:?}", err);
                ExitCode::FAILURE
            }
            Ok(()) => ExitCode::SUCCESS,
        }
    }
}

fn run_file(file_path: &String) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let mut interpreter = Interpreter::new();
    run(contents, &mut interpreter);
    Ok(())
}

fn run_prompt() -> Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        stdout().flush()?;

        let mut line = String::new();
        stdin().read_line(&mut line)?;
        let mut line = line.trim().to_string();
        if !line.ends_with(";") {
            line = format!("print {};", line);
        }
        run(line, &mut interpreter);
    }
}

fn run(source: String, interpreter: &mut Interpreter) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    //println!("{:#?}", tokens);
    let mut parser = Parser::new(tokens);
    let mut stmts = parser.parse();
    //println!("{:#?}", stmts);
    if parser.get_num_of_parser_errors() == 0 {
        let mut resolver = Resolver::new();
        resolver.resolve_stmts(&mut stmts);
        if resolver.get_num_of_resolver_errs() == 0 {
            println!("{:#?}", stmts);
            if let Err(err) = interpreter.interpret(&stmts) {
                error_at_runtime(err);
            }
        }
    }
}
