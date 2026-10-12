#[derive(Debug, PartialEq)]
pub enum Token {
    Integer(i64),
    Float(f64),
    Identifier(String),
    Text(String),
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    Comma,
    Newline,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        let token = match ch {
            ' ' | '\t' | '\r' => continue,
            '\n' => Token::Newline,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '=' => Token::Equal,
            ',' => Token::Comma,
            '0'..='9' => {
                let mut number = String::from(ch);

                while let Some(next) = chars.peek() {
                    if !next.is_ascii_digit() {
                        break;
                    }
                    number.push(chars.next().unwrap());
                }

                if chars.peek() == Some(&'.') {
                    number.push(chars.next().unwrap());

                    if !chars.peek().is_some_and(|next| next.is_ascii_digit()) {
                        return Err("Ondalık noktadan sonra bir rakam bekleniyor.".into());
                    }

                    while let Some(next) = chars.peek() {
                        if !next.is_ascii_digit() {
                            break;
                        }
                        number.push(chars.next().unwrap());
                    }

                    let value: f64 = number
                        .parse()
                        .map_err(|_| format!("Geçersiz ondalıklı sayı: {number}"))?;
                    if !value.is_finite() {
                        return Err(format!("Ondalıklı sayı çok büyük: {number}"));
                    }
                    Token::Float(value)
                } else {
                    let value = number
                        .parse()
                        .map_err(|_| format!("Tam sayı desteklenen aralığın dışında: {number}"))?;
                    Token::Integer(value)
                }
            }
            '"' => {
                let mut text = String::new();

                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some('n') => text.push('\n'),
                            Some('t') => text.push('\t'),
                            Some('"') => text.push('"'),
                            Some('\\') => text.push('\\'),
                            Some(other) => {
                                return Err(format!("Geçersiz kaçış dizisi: \\{other}"));
                            }
                            None => return Err("Metnin kapanış tırnağı eksik.".into()),
                        },
                        Some('\n' | '\r') => {
                            return Err("Metin satır bitmeden kapatılmalı.".into());
                        }
                        Some(other) => text.push(other),
                        None => return Err("Metnin kapanış tırnağı eksik.".into()),
                    }
                }

                Token::Text(text)
            }
            ch if ch.is_alphabetic() || ch == '_' => {
                let mut name = String::from(ch);

                while let Some(next) = chars.peek() {
                    if !next.is_alphanumeric() && *next != '_' {
                        break;
                    }
                    name.push(chars.next().unwrap());
                }

                Token::Identifier(name)
            }
            other => return Err(format!("Tanınmayan karakter: {other}")),
        };

        tokens.push(token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{Token, tokenize};

    #[test]
    fn tokenizes_function_call() {
        assert_eq!(
            tokenize("yaz(10 + 20)"),
            Ok(vec![
                Token::Identifier("yaz".into()),
                Token::LeftParen,
                Token::Integer(10),
                Token::Plus,
                Token::Integer(20),
                Token::RightParen,
            ])
        );
    }

    #[test]
    fn tokenizes_assignment_unicode_and_decimals() {
        assert_eq!(
            tokenize("öğrenci_sayısı2 = -12\r\noran = 1.25"),
            Ok(vec![
                Token::Identifier("öğrenci_sayısı2".into()),
                Token::Equal,
                Token::Minus,
                Token::Integer(12),
                Token::Newline,
                Token::Identifier("oran".into()),
                Token::Equal,
                Token::Float(1.25),
            ])
        );
    }

    #[test]
    fn tokenizes_text_and_escapes() {
        assert_eq!(
            tokenize(r#"yaz("Merhaba, dünya!\n\t\"Kıvıl\"\\", isim)"#),
            Ok(vec![
                Token::Identifier("yaz".into()),
                Token::LeftParen,
                Token::Text("Merhaba, dünya!\n\t\"Kıvıl\"\\".into()),
                Token::Comma,
                Token::Identifier("isim".into()),
                Token::RightParen,
            ])
        );
    }

    #[test]
    fn tokenizes_arithmetic_and_whitespace() {
        assert_eq!(
            tokenize(" \t(8 - 2) * 3 / 2 "),
            Ok(vec![
                Token::LeftParen,
                Token::Integer(8),
                Token::Minus,
                Token::Integer(2),
                Token::RightParen,
                Token::Star,
                Token::Integer(3),
                Token::Slash,
                Token::Integer(2),
            ])
        );
        assert_eq!(tokenize(""), Ok(vec![]));
    }

    #[test]
    fn rejects_invalid_input() {
        for source in [
            "@",
            "\"kapanmamış",
            "\"satır\nsonu\"",
            r#""hatalı\q""#,
            "\"ters eğik çizgi\\",
            "1.",
            "1.2.3",
            "9223372036854775808",
        ] {
            assert!(tokenize(source).is_err(), "Hata bekleniyordu: {source}");
        }
        assert!(tokenize(&format!("{}.0", "9".repeat(400))).is_err());
    }
}
