mod interpreter;
mod lexer;
mod parser;

fn main() -> std::process::ExitCode {
    let source = r#"isim = "Herdem"
puan = 10
puan = puan + 5
yaz("Merhaba", isim)
yaz("Puanın:", puan)
mesaj = "Merhaba " + isim
yaz(mesaj)"#;

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
    let statements = parser::parse(&tokens)?;
    let mut interpreter = interpreter::Interpreter::default();

    for statement in &statements {
        if let Some(result) = interpreter.execute(statement)? {
            println!("{result}");
        }
    }

    Ok(())
}
