mod interpreter;
mod lexer;
mod parser;

fn main() -> std::process::ExitCode {
    let source = "10 + 20 * 3\n(10 + 20) * 3\n-3 * (2 + 4)\n5 / 2";

    match run(source) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Hata: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(source: &str) -> Result<(), String> {
    let tokens = lexer::tokenize(source)?;
    let expressions = parser::parse(&tokens)?;

    for expression in expressions {
        let result = interpreter::evaluate(&expression)?;
        println!("{result}");
    }

    Ok(())
}
