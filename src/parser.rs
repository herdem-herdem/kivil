use crate::diagnostic::{Diagnostic, Position};
use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Located(Box<Expr>, Position),
    Number(f64),
    Boolean(bool),
    Not(Box<Expr>),
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
    Located(Box<Statement>, Position),
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    Assignment {
        name: String,
        value: Expr,
    },
    Print(Vec<Expr>),
    Expression(Expr),
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    And,
    Or,
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
    parser.sequence(false)
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

    fn sequence(&mut self, in_block: bool) -> Result<Vec<Statement>, Diagnostic> {
        let mut statements = Vec::new();
        loop {
            while self.peek() == Some(&TokenKind::Newline) {
                self.current += 1;
            }
            if self.peek().is_none() || self.peek() == Some(&TokenKind::Eof) {
                break;
            }
            if in_block && self.peek() == Some(&TokenKind::RightBrace) {
                break;
            }
            statements.push(self.statement()?);
            let at_end = matches!(self.peek(), Some(TokenKind::Newline | TokenKind::Eof))
                || (in_block && self.peek() == Some(&TokenKind::RightBrace));
            if !at_end {
                return Err(Diagnostic::from("Komut sonu bekleniyor.").at(self.position()));
            }
        }
        Ok(statements)
    }

    fn block(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        if self.peek() != Some(&TokenKind::LeftBrace) {
            return Err(Diagnostic::from("Blok başlangıcı '{' bekleniyor.").at(self.position()));
        }
        self.current += 1;
        let body = self.sequence(true)?;
        if self.peek() != Some(&TokenKind::RightBrace) {
            return Err(Diagnostic::from("Blok sonu '}' bekleniyor.").at(self.position()));
        }
        self.current += 1;
        Ok(body)
    }

    fn statement(&mut self) -> Result<Statement, Diagnostic> {
        let position = self.position();
        self.statement_value()
            .map(|statement| Statement::Located(Box::new(statement), position))
            .map_err(|error| error.at(self.position()))
    }

    fn statement_value(&mut self) -> Result<Statement, Diagnostic> {
        if self.peek() == Some(&TokenKind::If) {
            self.current += 1;
            if self.peek() != Some(&TokenKind::LeftParen) {
                return Err(
                    Diagnostic::from("eger koşulundan önce '(' bekleniyor.").at(self.position())
                );
            }
            self.current += 1;
            let condition = self.expression()?;
            if self.peek() != Some(&TokenKind::RightParen) {
                return Err(
                    Diagnostic::from("eger koşulundan sonra ')' bekleniyor.").at(self.position())
                );
            }
            self.current += 1;
            let then_body = self.block()?;
            let after_block = self.current;
            while self.peek() == Some(&TokenKind::Newline) {
                self.current += 1;
            }
            let else_body = if self.peek() == Some(&TokenKind::Else) {
                self.current += 1;
                if self.peek() == Some(&TokenKind::If) {
                    vec![self.statement()?]
                } else {
                    self.block()?
                }
            } else {
                self.current = after_block;
                Vec::new()
            };
            return Ok(Statement::If {
                condition,
                then_body,
                else_body,
            });
        }

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
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.logical_and()?;
        while self.peek() == Some(&TokenKind::Or) {
            let position = self.position();
            self.current += 1;
            let right = self.logical_and()?;
            left = Expr::Located(
                Box::new(Expr::Binary {
                    left: Box::new(left),
                    operator: Operator::Or,
                    right: Box::new(right),
                }),
                position,
            );
        }
        Ok(left)
    }

    fn logical_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.logical_not()?;
        while self.peek() == Some(&TokenKind::And) {
            let position = self.position();
            self.current += 1;
            let right = self.logical_not()?;
            left = Expr::Located(
                Box::new(Expr::Binary {
                    left: Box::new(left),
                    operator: Operator::And,
                    right: Box::new(right),
                }),
                position,
            );
        }
        Ok(left)
    }

    fn logical_not(&mut self) -> Result<Expr, Diagnostic> {
        if self.peek() == Some(&TokenKind::Not) {
            let position = self.position();
            self.current += 1;
            Ok(Expr::Located(
                Box::new(Expr::Not(Box::new(self.logical_not()?))),
                position,
            ))
        } else {
            self.equality()
        }
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
    fn strip_statement(statement: Statement) -> Statement {
        match statement {
            Statement::Located(statement, _) => strip_statement(*statement),
            Statement::Assignment { name, value } => Statement::Assignment {
                name,
                value: strip(value),
            },
            Statement::Print(args) => Statement::Print(args.into_iter().map(strip).collect()),
            Statement::Expression(expr) => Statement::Expression(strip(expr)),
            other => other,
        }
    }
    fn parse(tokens: &[crate::lexer::Token]) -> Result<Vec<Statement>, String> {
        super::parse(tokens)
            .map(|statements| statements.into_iter().map(strip_statement).collect())
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

    #[test]
    fn requires_parentheses_around_if_and_else_if_conditions() {
        for source in [
            "eger (dogru) {}",
            "eger ((1 + 2) > 2) { 1 } degilse { 2 }",
            "eger (yanlis) {} degilse eger (dogru) {}",
        ] {
            assert!(parse(&tokenize(source).unwrap()).is_ok(), "{source}");
        }
        for source in [
            "eger dogru {}",
            "eger (yanlis) {} degilse eger dogru {}",
            "eger () {}",
            "eger (dogru {}",
            "eger (dogru) veya yanlis {}",
        ] {
            assert!(parse(&tokenize(source).unwrap()).is_err(), "{source}");
        }
    }
}
