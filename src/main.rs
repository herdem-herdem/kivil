mod interpreter;
mod lexer;
mod parser;

fn main() -> std::process::ExitCode {
    match run_file() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Hata: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_file() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1);
    let path = arguments.next().ok_or("Kullanım: kivil <dosya.kivil>")?;
    if arguments.next().is_some() {
        return Err("Kullanım: kivil <dosya.kivil>".into());
    }
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("Dosya okunamadı ({}): {error}", path.to_string_lossy()))?;
    run(&source)
}

fn run(source: &str) -> Result<(), String> {
    let tokens = lexer::tokenize(source)?;
    let statements = parser::parse(&tokens)?;
    let mut interpreter = interpreter::Interpreter::default();

    for statement in &statements {
        if let Some(result) = interpreter.execute(statement)? {
            println!("{result}");
        }
    }

    Ok(())
}
