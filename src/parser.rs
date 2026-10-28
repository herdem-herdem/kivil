use crate::diagnostic::{Diagnostic, Position};
use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Located(Box<Expr>, Position),
    Number(f64),
    Boolean(bool),
    Text(String),
    Variable(String),
    Positive(Box<Expr>),
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
    Print(Vec<Expr>),
    Expression(Expr),
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Statement>, Diagnostic> {
    let mut parser = Parser { tokens, current: 0 };
    let mut statements = Vec::new();

    while parser.peek() != Some(&TokenKind::Eof) && parser.peek().is_some() {
        if parser.peek() == Some(&TokenKind::Newline) {
            parser.current += 1;
            continue;
        }

        statements.push(
            parser
                .statement()
                .map_err(|error| error.at(parser.position()))?,
        );

        if let Some(token) = parser.peek()
            && *token != TokenKind::Newline
            && *token != TokenKind::Eof
        {
            return Err(
                Diagnostic::from(format!("İfade sonu bekleniyordu, bulunan: {token:?}"))
                    .at(parser.position()),
            );
        }
    }

    Ok(statements)
}

struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.current).map(|token| &token.kind)
    }

    fn position(&self) -> Position {
        self.tokens
            .get(self.current)
            .or_else(|| self.tokens.last())
            .map(|token| token.position)
            .unwrap_or(Position { line: 1, column: 1 })
    }

    fn statement(&mut self) -> Result<Statement, Diagnostic> {
        if let Some(TokenKind::Identifier(name)) = self.peek()
            && name == "yaz"
            && self.tokens.get(self.current + 1).map(|token| &token.kind)
                == Some(&TokenKind::LeftParen)
        {
            self.current += 2;
            let mut arguments = Vec::new();

            if self.peek() != Some(&TokenKind::RightParen) {
                loop {
                    arguments.push(self.expression()?);
                    if self.peek() != Some(&TokenKind::Comma) {
                        break;
                    }
                    self.current += 1;
                }
            }

            if self.peek() != Some(&TokenKind::RightParen) {
                return Err("yaz çağrısında ',' veya ')' bekleniyor.".into());
            }
            self.current += 1;
            return Ok(Statement::Print(arguments));
        }

        if let Some(TokenKind::Identifier(name)) = self.peek()
            && self.tokens.get(self.current + 1).map(|token| &token.kind) == Some(&TokenKind::Equal)
        {
            let name = name.clone();
            self.current += 2;
            let value = self.expression()?;
            return Ok(Statement::Assignment { name, value });
        }

        Ok(Statement::Expression(self.expression()?))
    }

    fn expression(&mut self) -> Result<Expr, Diagnostic> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.comparison()?;
        loop {
            let operator = match self.peek() {
                Some(TokenKind::EqualEqual) => Operator::Equal,
                Some(TokenKind::NotEqual) => Operator::NotEqual,
                _ => break,
            };
            let position = self.position();
            self.current += 1;
            let right = self.comparison()?;
            left = Expr::Located(
                Box::new(Expr::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                }),
                position,
            );
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.addition()?;
        loop {
            let operator = match self.peek() {
                Some(TokenKind::Less) => Operator::Less,
                Some(TokenKind::LessEqual) => Operator::LessEqual,
                Some(TokenKind::Greater) => Operator::Greater,
                Some(TokenKind::GreaterEqual) => Operator::GreaterEqual,
                _ => break,
            };
            let position = self.position();
            self.current += 1;
            let right = self.addition()?;
            left = Expr::Located(
                Box::new(Expr::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                }),
                position,
            );
        }
        Ok(left)
    }

    fn addition(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.term()?;

        loop {
            let operator = match self.peek() {
                Some(TokenKind::Plus) => Operator::Add,
                Some(TokenKind::Minus) => Operator::Subtract,
                _ => break,
            };
            let position = self.position();
            self.current += 1;
            let right = self.term()?;

            left = Expr::Located(
                Box::new(Expr::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                }),
                position,
            );
        }

        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.unary()?;

        loop {
            let operator = match self.peek() {
                Some(TokenKind::Star) => Operator::Multiply,
                Some(TokenKind::Slash) => Operator::Divide,
                _ => break,
            };
            let position = self.position();
            self.current += 1;
            let right = self.unary()?;

            left = Expr::Located(
                Box::new(Expr::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                }),
                position,
            );
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, Diagnostic> {
        let position = self.position();
        match self.peek() {
            Some(TokenKind::Minus) => {
                self.current += 1;
                Ok(Expr::Located(
                    Box::new(Expr::Negate(Box::new(self.unary()?))),
                    position,
                ))
            }
            Some(TokenKind::Plus) => {
                self.current += 1;
                Ok(Expr::Located(
                    Box::new(Expr::Positive(Box::new(self.unary()?))),
                    position,
                ))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let position = self.position();
        self.primary_value()
            .map(|expression| Expr::Located(Box::new(expression), position))
            .map_err(|error| error.at(position))
    }

    fn primary_value(&mut self) -> Result<Expr, Diagnostic> {
        match self.peek() {
            Some(TokenKind::True) => {
                self.current += 1;
                Ok(Expr::Boolean(true))
            }
            Some(TokenKind::False) => {
                self.current += 1;
                Ok(Expr::Boolean(false))
            }
            Some(TokenKind::Text(text)) => {
                let text = text.clone();
                self.current += 1;
                Ok(Expr::Text(text))
            }
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.current += 1;
                Ok(Expr::Variable(name))
            }
            Some(TokenKind::Integer(value)) => {
                let value = *value as f64;
                self.current += 1;
                Ok(Expr::Number(value))
            }
            Some(TokenKind::Float(value)) => {
                let value = *value;
                self.current += 1;
                Ok(Expr::Number(value))
            }
            Some(TokenKind::LeftParen) => {
                self.current += 1;
                let expression = self.expression()?;

                if self.peek() != Some(&TokenKind::RightParen) {
                    return Err(
                        Diagnostic::from("Kapanış parantezi ')' bekleniyor.").at(self.position())
                    );
                }
                self.current += 1;

                Ok(expression)
            }
            Some(token) => Err(format!(
                "Sayı, metin, değişken adı veya '(' bekleniyordu, bulunan: {token:?}"
            )
            .into()),
            None => {
                Err("İfade tamamlanmamış; sayı, metin, değişken adı veya '(' bekleniyor.".into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Operator, Statement};
    fn strip(expression: Expr) -> Expr {
        match expression {
            Expr::Located(inner, _) => strip(*inner),
            Expr::Binary {
                left,
                operator,
                right,
            } => Expr::Binary {
                left: Box::new(strip(*left)),
                operator,
                right: Box::new(strip(*right)),
            },
            Expr::Negate(inner) => Expr::Negate(Box::new(strip(*inner))),
            Expr::Positive(inner) => Expr::Positive(Box::new(strip(*inner))),
            other => other,
        }
    }
    fn parse(tokens: &[crate::lexer::Token]) -> Result<Vec<Statement>, String> {
        super::parse(tokens)
            .map(|statements| {
                statements
                    .into_iter()
                    .map(|statement| match statement {
                        Statement::Assignment { name, value } => Statement::Assignment {
                            name,
                            value: strip(value),
                        },
                        Statement::Print(args) => {
                            Statement::Print(args.into_iter().map(strip).collect())
                        }
                        Statement::Expression(expr) => Statement::Expression(strip(expr)),
                    })
                    .collect()
            })
            .map_err(|error| error.to_string())
    }
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
            "yaz(1 2)",
            "yaz(1,)",
            "yaz(,1)",
            "yaz(1",
            "yaz(1\n2)",
            "sonuc = yaz(1)",
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

    #[test]
    fn parses_text_and_print_arguments() {
        assert_eq!(
            parse(&tokenize("isim = \"Herdem\"\nyaz(\"Merhaba\", isim)\nyaz()").unwrap()),
            Ok(vec![
                Statement::Assignment {
                    name: "isim".into(),
                    value: Expr::Text("Herdem".into()),
                },
                Statement::Print(vec![
                    Expr::Text("Merhaba".into()),
                    Expr::Variable("isim".into())
                ]),
                Statement::Print(vec![]),
            ])
        );
    }
}
