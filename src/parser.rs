use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(f64),
    Variable(String),
    Negate(Box<Expr>),
    Binary {
        left: Box<Expr>,
        operator: Operator,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Assignment { name: String, value: Expr },
    Expression(Expr),
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Statement>, String> {
    let mut parser = Parser { tokens, current: 0 };
    let mut statements = Vec::new();

    while parser.peek().is_some() {
        if parser.peek() == Some(&Token::Newline) {
            parser.current += 1;
            continue;
        }

        statements.push(parser.statement()?);

        if let Some(token) = parser.peek()
            && *token != Token::Newline
        {
            return Err(format!("İfade sonu bekleniyordu, bulunan: {token:?}"));
        }
    }

    Ok(statements)
}

struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn statement(&mut self) -> Result<Statement, String> {
        if let Some(Token::Identifier(name)) = self.peek()
            && self.tokens.get(self.current + 1) == Some(&Token::Equal)
        {
            let name = name.clone();
            self.current += 2;
            let value = self.expression()?;
            return Ok(Statement::Assignment { name, value });
        }

        Ok(Statement::Expression(self.expression()?))
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
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.current += 1;
                Ok(Expr::Variable(name))
            }
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
            Some(token) => Err(format!(
                "Sayı, değişken adı veya '(' bekleniyordu, bulunan: {token:?}"
            )),
            None => Err("İfade tamamlanmamış; sayı, değişken adı veya '(' bekleniyor.".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Operator, Statement, parse};
    use crate::lexer::tokenize;

    #[test]
    fn multiplication_binds_more_tightly_than_addition() {
        assert_eq!(
            parse(&tokenize("10 + 20 * 3").unwrap()),
            Ok(vec![Statement::Expression(Expr::Binary {
                left: Box::new(Expr::Number(10.0)),
                operator: Operator::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(20.0)),
                    operator: Operator::Multiply,
                    right: Box::new(Expr::Number(3.0)),
                }),
            })])
        );
    }

    #[test]
    fn separates_lines_and_ignores_blank_lines() {
        assert_eq!(
            parse(&tokenize("\n10\n\n20\n").unwrap()),
            Ok(vec![
                Statement::Expression(Expr::Number(10.0)),
                Statement::Expression(Expr::Number(20.0)),
            ])
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
            "puan =",
            "puan =\n10",
            "10 = 20",
            "puan + 1 = 20",
            "(puan) = 20",
            "a = b = 10",
            "yaz(10)",
            "\"metin\"",
        ] {
            assert!(
                parse(&tokenize(source).unwrap()).is_err(),
                "Hata bekleniyordu: {source}"
            );
        }
    }

    #[test]
    fn distinguishes_assignment_from_variable_expression() {
        assert_eq!(
            parse(&tokenize("puan = 10\npuan").unwrap()),
            Ok(vec![
                Statement::Assignment {
                    name: "puan".into(),
                    value: Expr::Number(10.0),
                },
                Statement::Expression(Expr::Variable("puan".into())),
            ])
        );
    }
}
