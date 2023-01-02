use jlox::scanner::Scanner;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::process::ExitCode;
use std::{env, io};

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

fn run_file(file_path: &String) -> io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    run(contents);

    Ok(())
}

fn run_prompt() -> io::Result<()> {
    loop {
        print!("> ");
        stdout().flush()?;

        let mut line = String::new();
        stdin().read_line(&mut line)?;
        let line = line.trim().to_string();
        run(line)
    }
}

fn run(source: String) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    for token in tokens {
        println!("{}", token)
    }
}
