mod lexer;

fn main() {
    let source = "yaz(10 + 20)";

    match lexer::tokenize(source) {
        Ok(tokens) => println!("{tokens:#?}"),
        Err(message) => eprintln!("Lexer hatası: {message}"),
    }
}
