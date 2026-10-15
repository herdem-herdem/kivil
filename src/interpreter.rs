use std::collections::HashMap;

use crate::parser::{Expr, Operator, Statement};

#[derive(Default)]
pub struct Interpreter {
    variables: HashMap<String, f64>,
}

impl Interpreter {
    pub fn execute(&mut self, statement: &Statement) -> Result<Option<f64>, String> {
        match statement {
            Statement::Assignment { name, value } => {
                let value = self.evaluate(value)?;
                self.variables.insert(name.clone(), value);
                Ok(None)
            }
            Statement::Expression(expression) => Ok(Some(self.evaluate(expression)?)),
        }
    }

    fn evaluate(&self, expression: &Expr) -> Result<f64, String> {
        let result = match expression {
            Expr::Number(value) => *value,
            Expr::Variable(name) => *self
                .variables
                .get(name)
                .ok_or_else(|| format!("Tanımlanmamış değişken: {name}"))?,
            Expr::Negate(operand) => -self.evaluate(operand)?,
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator {
                    Operator::Add => left + right,
                    Operator::Subtract => left - right,
                    Operator::Multiply => left * right,
                    Operator::Divide => {
                        if right == 0.0 {
                            return Err("Sıfıra bölme yapılamaz.".into());
                        }
                        left / right
                    }
                }
            }
        };

        if !result.is_finite() {
            return Err("İşlem sonucu desteklenen sayı aralığının dışında.".into());
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::Interpreter;
    use crate::{lexer::tokenize, parser::parse};

    fn calculate(source: &str) -> Result<Vec<f64>, String> {
        let tokens = tokenize(source)?;
        let statements = parse(&tokens)?;
        let mut interpreter = Interpreter::default();
        let mut results = Vec::new();

        for statement in &statements {
            if let Some(value) = interpreter.execute(statement)? {
                results.push(value);
            }
        }

        Ok(results)
    }

    #[test]
    fn evaluates_precedence_parentheses_and_left_associativity() {
        assert_eq!(
            calculate("10 + 20 * 3\n(10 + 20) * 3\n20 - 5 - 3\n24 / 4 / 2"),
            Ok(vec![70.0, 90.0, 12.0, 3.0])
        );
    }

    #[test]
    fn evaluates_signs_decimals_and_nested_parentheses() {
        assert_eq!(
            calculate("-3 * (2 + 4)\n5 / 2\n1.5 + 2\n--3\n+2\n2 * -3\n1 - -2\n((2 + 3) * 4)"),
            Ok(vec![-18.0, 2.5, 3.5, 3.0, 2.0, -6.0, 3.0, 20.0])
        );
    }

    #[test]
    fn rejects_division_by_zero_and_overflow() {
        for source in ["1 / 0", "1 / (2 - 2)", "1 / -0.0"] {
            assert_eq!(calculate(source), Err("Sıfıra bölme yapılamaz.".into()));
        }

        let large_number = format!("{}.0", "9".repeat(200));
        assert!(calculate(&format!("{large_number} * {large_number}")).is_err());
    }

    #[test]
    fn stores_variables_and_evaluates_reassignments() {
        assert_eq!(
            calculate(
                "puan = 10\nbonus = 20\ntoplam = puan + bonus * 3\ntoplam\npuan = puan + 5\npuan\ntoplam"
            ),
            Ok(vec![70.0, 15.0, 70.0])
        );
        assert_eq!(calculate("puan = 10"), Ok(vec![]));
    }

    #[test]
    fn supports_unicode_names_and_decimals() {
        assert_eq!(
            calculate("öğrenci_sayısı = 3\n_oran2 = 1.5\n-(öğrenci_sayısı + _oran2) / 2"),
            Ok(vec![-2.25])
        );
    }

    #[test]
    fn reports_undefined_variables() {
        for source in ["puan", "puan + 1", "puan = puan + 1", "sonuc = eksik * 2"] {
            let name = if source.contains("eksik") {
                "eksik"
            } else {
                "puan"
            };
            assert_eq!(
                calculate(source),
                Err(format!("Tanımlanmamış değişken: {name}"))
            );
        }
    }

    #[test]
    fn failed_assignment_preserves_the_previous_value_and_stops_execution() {
        let statements = parse(&tokenize("puan = 10\npuan = 1 / 0\npuan = 99").unwrap()).unwrap();
        let mut interpreter = Interpreter::default();

        assert_eq!(interpreter.execute(&statements[0]), Ok(None));
        assert!(interpreter.execute(&statements[1]).is_err());
        let read = parse(&tokenize("puan").unwrap()).unwrap();
        assert_eq!(interpreter.execute(&read[0]), Ok(Some(10.0)));

        assert_eq!(
            calculate("puan = 10\npuan = 1 / 0\npuan = 99"),
            Err("Sıfıra bölme yapılamaz.".into())
        );
    }
}
