use std::collections::HashMap;
use std::fmt;

use crate::parser::{Expr, Operator, Statement};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(f64),
    Text(String),
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(value) => write!(formatter, "{value}"),
            Value::Text(text) => write!(formatter, "{text}"),
        }
    }
}

#[derive(Default)]
pub struct Interpreter {
    variables: HashMap<String, Value>,
}

impl Interpreter {
    pub fn execute(&mut self, statement: &Statement) -> Result<Option<Value>, String> {
        match statement {
            Statement::Assignment { name, value } => {
                let value = self.evaluate(value)?;
                self.variables.insert(name.clone(), value);
                Ok(None)
            }
            Statement::Print(arguments) => {
                let mut parts = Vec::new();
                for argument in arguments {
                    parts.push(self.evaluate(argument)?.to_string());
                }
                Ok(Some(Value::Text(parts.join(" "))))
            }
            Statement::Expression(expression) => Ok(Some(self.evaluate(expression)?)),
        }
    }

    fn evaluate(&self, expression: &Expr) -> Result<Value, String> {
        let result = match expression {
            Expr::Number(value) => Value::Number(*value),
            Expr::Text(text) => Value::Text(text.clone()),
            Expr::Variable(name) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Tanımlanmamış değişken: {name}"))?,
            Expr::Negate(operand) => match self.evaluate(operand)? {
                Value::Number(value) => Value::Number(-value),
                Value::Text(_) => return Err("'-' işareti yalnızca sayılara uygulanabilir.".into()),
            },
            Expr::Positive(operand) => match self.evaluate(operand)? {
                Value::Number(value) => Value::Number(value),
                Value::Text(_) => return Err("'+' işareti yalnızca sayılara uygulanabilir.".into()),
            },
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match (left, operator, right) {
                    (Value::Number(left), Operator::Add, Value::Number(right)) => {
                        Value::Number(left + right)
                    }
                    (Value::Text(left), Operator::Add, Value::Text(right)) => {
                        Value::Text(left + &right)
                    }
                    (Value::Number(left), Operator::Subtract, Value::Number(right)) => {
                        Value::Number(left - right)
                    }
                    (Value::Number(left), Operator::Multiply, Value::Number(right)) => {
                        Value::Number(left * right)
                    }
                    (Value::Number(left), Operator::Divide, Value::Number(right)) => {
                        if right == 0.0 {
                            return Err("Sıfıra bölme yapılamaz.".into());
                        }
                        Value::Number(left / right)
                    }
                    (_, Operator::Add, _) => {
                        return Err("'+' için iki sayı veya iki metin gerekiyor.".into());
                    }
                    _ => {
                        return Err(
                            "Çıkarma, çarpma ve bölme yalnızca sayılarla yapılabilir.".into()
                        );
                    }
                }
            }
        };

        if let Value::Number(value) = &result
            && !value.is_finite()
        {
            return Err("İşlem sonucu desteklenen sayı aralığının dışında.".into());
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{Interpreter, Value};
    use crate::{lexer::tokenize, parser::parse};

    fn calculate(source: &str) -> Result<Vec<Value>, String> {
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

    fn numbers(values: &[f64]) -> Result<Vec<Value>, String> {
        Ok(values.iter().copied().map(Value::Number).collect())
    }

    #[test]
    fn evaluates_precedence_parentheses_and_left_associativity() {
        assert_eq!(
            calculate("10 + 20 * 3\n(10 + 20) * 3\n20 - 5 - 3\n24 / 4 / 2"),
            numbers(&[70.0, 90.0, 12.0, 3.0])
        );
    }

    #[test]
    fn evaluates_signs_decimals_and_nested_parentheses() {
        assert_eq!(
            calculate("-3 * (2 + 4)\n5 / 2\n1.5 + 2\n--3\n+2\n2 * -3\n1 - -2\n((2 + 3) * 4)"),
            numbers(&[-18.0, 2.5, 3.5, 3.0, 2.0, -6.0, 3.0, 20.0])
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
            numbers(&[70.0, 15.0, 70.0])
        );
        assert_eq!(calculate("puan = 10"), Ok(vec![]));
    }

    #[test]
    fn supports_unicode_names_and_decimals() {
        assert_eq!(
            calculate("öğrenci_sayısı = 3\n_oran2 = 1.5\n-(öğrenci_sayısı + _oran2) / 2"),
            numbers(&[-2.25])
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
        assert_eq!(interpreter.execute(&read[0]), Ok(Some(Value::Number(10.0))));

        assert_eq!(
            calculate("puan = 10\npuan = 1 / 0\npuan = 99"),
            Err("Sıfıra bölme yapılamaz.".into())
        );
    }

    #[test]
    fn stores_and_concatenates_text() {
        assert_eq!(
            calculate(
                "isim = \"Herdem\"\nmesaj = \"Merhaba \" + isim\nmesaj\nisim = isim + \"!\"\nisim\nmesaj"
            ),
            Ok(vec![
                Value::Text("Merhaba Herdem".into()),
                Value::Text("Herdem!".into()),
                Value::Text("Merhaba Herdem".into()),
            ])
        );
    }

    #[test]
    fn prints_text_numbers_and_calculated_arguments() {
        assert_eq!(
            calculate(
                "isim = \"Herdem\"\npuan = 10\npuan = puan + 5\nyaz(\"Merhaba\", isim)\nyaz(\"Puanın:\", puan)\nyaz((2 + 3) * 4, 5 / 2)\nyaz()\nyaz(\"ilk\\nikinci\")"
            ),
            Ok(vec![
                Value::Text("Merhaba Herdem".into()),
                Value::Text("Puanın: 15".into()),
                Value::Text("20 2.5".into()),
                Value::Text(String::new()),
                Value::Text("ilk\nikinci".into()),
            ])
        );
    }

    #[test]
    fn rejects_invalid_operand_types() {
        for source in [
            "\"Puan: \" + 15",
            "15 + \"puan\"",
            "\"a\" - \"b\"",
            "\"a\" * 3",
            "3 / \"a\"",
            "-\"a\"",
            "+\"a\"",
        ] {
            assert!(calculate(source).is_err(), "Hata bekleniyordu: {source}");
        }
    }

    #[test]
    fn print_reports_argument_errors_without_returning_partial_output() {
        assert_eq!(
            calculate("yaz(\"Puan:\", eksik)"),
            Err("Tanımlanmamış değişken: eksik".into())
        );
        assert!(calculate("yaz(\"Puan:\", 1 / 0)").is_err());
    }

    #[test]
    fn reassignment_can_change_type_and_failed_assignment_preserves_text() {
        assert_eq!(
            calculate("deger = 10\ndeger = \"on\"\ndeger"),
            Ok(vec![Value::Text("on".into())])
        );

        let statements =
            parse(&tokenize("isim = \"Herdem\"\nisim = isim + 1\nisim").unwrap()).unwrap();
        let mut interpreter = Interpreter::default();
        assert_eq!(interpreter.execute(&statements[0]), Ok(None));
        assert!(interpreter.execute(&statements[1]).is_err());
        assert_eq!(
            interpreter.execute(&statements[2]),
            Ok(Some(Value::Text("Herdem".into())))
        );
    }
}
