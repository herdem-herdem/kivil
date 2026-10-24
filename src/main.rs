mod diagnostic;
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
    run(&source).map_err(|error| format!("{}:{error}", path.to_string_lossy()))
}

fn run(source: &str) -> Result<(), diagnostic::Diagnostic> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Position;

    #[test]
    fn reports_unicode_columns_and_runtime_locations() {
        let error = run("isim = \"İ\"\n  10 / 0").unwrap_err();
        assert_eq!(error.position, Some(Position { line: 2, column: 6 }));
        let error = lexer::tokenize("öğrenci @").unwrap_err();
        assert_eq!(error.position, Some(Position { line: 1, column: 9 }));
        let error = run("1 +").unwrap_err();
        assert_eq!(error.position, Some(Position { line: 1, column: 4 }));
    }

    #[test]
    fn comments_preserve_newlines_and_hashes_inside_strings() {
        assert!(run("# başlık\r\nisim = \"# metin\" # açıklama\r\nyaz(isim) # son").is_ok());
        let error = run("1\n# açıklama\neksik").unwrap_err();
        assert_eq!(error.position, Some(Position { line: 3, column: 1 }));
    }
}
