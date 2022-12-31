use jlox::token::{Token, TokenType};
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::process::ExitCode;
use std::{env, io};

fn main() -> ExitCode {
    let s = "Some (\"word\")";
    println!("{s}");
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        eprintln!("Usage: jlox [script]");
        return ExitCode::from(ExitCode::FAILURE);
    } else if args.len() == 1 {
        runFile(&args[1]);
    } else {
        runPrompt();
    }
    ExitCode::SUCCESS
}

fn runFile(filePath: &String) -> io::Result<()> {
    let mut file = File::open(filePath)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    run(contents);

    Ok(())
}

fn runPrompt() -> io::Result<()> {
    loop {
        println!("> ");

        let mut line = String::new();
        stdin().read_line(&mut line)?;
        let line = line.trim().to_string();
        run(line)
    }
}

fn run(source: String) {}
