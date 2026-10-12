use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(f64),
    Negate(Box<Expr>),
    Binary {
        left: Box<Expr>,
        operator: Operator,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Expr>, String> {
    let mut parser = Parser { tokens, current: 0 };
    let mut expressions = Vec::new();

    while parser.peek().is_some() {
        if parser.peek() == Some(&Token::Newline) {
            parser.current += 1;
            continue;
        }

        expressions.push(parser.expression()?);

        if let Some(token) = parser.peek()
            && *token != Token::Newline
        {
            return Err(format!("İfade sonu bekleniyordu, bulunan: {token:?}"));
        }
    }

    Ok(expressions)
}

struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn expression(&mut self) -> Result<Expr, String> {
        let mut left = self.term()?;

        loop {
            let operator = match self.peek() {
                Some(Token::Plus) => Operator::Add,
                Some(Token::Minus) => Operator::Subtract,
                _ => break,
            };
            self.current += 1;
            let right = self.term()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;

        loop {
            let operator = match self.peek() {
                Some(Token::Star) => Operator::Multiply,
                Some(Token::Slash) => Operator::Divide,
                _ => break,
            };
            self.current += 1;
            let right = self.unary()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Minus) => {
                self.current += 1;
                Ok(Expr::Negate(Box::new(self.unary()?)))
            }
            Some(Token::Plus) => {
                self.current += 1;
                self.unary()
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Integer(value)) => {
                let value = *value as f64;
                self.current += 1;
                Ok(Expr::Number(value))
            }
            Some(Token::Float(value)) => {
                let value = *value;
                self.current += 1;
                Ok(Expr::Number(value))
            }
            Some(Token::LeftParen) => {
                self.current += 1;
                let expression = self.expression()?;

                if self.peek() != Some(&Token::RightParen) {
                    return Err("Kapanış parantezi ')' bekleniyor.".into());
                }
                self.current += 1;

                Ok(expression)
            }
            Some(token) => Err(format!("Sayı veya '(' bekleniyordu, bulunan: {token:?}")),
            None => Err("İfade tamamlanmamış; sayı veya '(' bekleniyor.".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Operator, parse};
    use crate::lexer::tokenize;

    #[test]
    fn multiplication_binds_more_tightly_than_addition() {
        assert_eq!(
            parse(&tokenize("10 + 20 * 3").unwrap()),
            Ok(vec![Expr::Binary {
                left: Box::new(Expr::Number(10.0)),
                operator: Operator::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(20.0)),
                    operator: Operator::Multiply,
                    right: Box::new(Expr::Number(3.0)),
                }),
            }])
        );
    }

    #[test]
    fn separates_lines_and_ignores_blank_lines() {
        assert_eq!(
            parse(&tokenize("\n10\n\n20\n").unwrap()),
            Ok(vec![Expr::Number(10.0), Expr::Number(20.0)])
        );
        assert_eq!(parse(&[]), Ok(vec![]));
    }

    #[test]
    fn rejects_incomplete_or_unsupported_expressions() {
        for source in [
            "10 +",
            "10 +\n20",
            "()",
            "(1 + 2",
            "1 + 2)",
            "1 2",
            "2(3)",
            "1 + * 2",
            "puan = 10",
            "yaz(10)",
            "\"metin\"",
        ] {
            assert!(
                parse(&tokenize(source).unwrap()).is_err(),
                "Hata bekleniyordu: {source}"
            );
        }
    }
}
