use crate::scrabble_base_types::ScrabbleLetter;

#[derive(Debug, Eq, PartialEq)]
pub struct Term {
    tokens: Vec<ScrabbleLetter>,
}

impl Term {
    pub fn new(letters: &Vec<ScrabbleLetter>) -> Term {
        Term {
            tokens: letters.clone(),
        }
    }

    pub fn is_singleton(&self) -> bool {
        return self.tokens.len() == 1;
    }

    pub fn evaluate(&self) -> Result<i32, String> {
        let mut operand_stack: Vec<i32> = Vec::new();
        for token in &self.tokens {
            match token {
                ScrabbleLetter::Plus => binary_operator(|f, s| f + s, "+", &mut operand_stack)?,
                ScrabbleLetter::Minus => binary_operator(|f, s| f - s, "-", &mut operand_stack)?,
                ScrabbleLetter::Dot => binary_operator(|f, s| f * s, "*", &mut operand_stack)?,
                ScrabbleLetter::Empty => return Err("Found empty token in term!".to_string()),
                num => operand_stack.push(*num as i32),
            }
        }

        if operand_stack.len() > 1 {
            return Err("Unused arguments are left on the stack!".to_string());
        }
        operand_stack
            .pop()
            .ok_or("Empty operand stack at the end of evaluation!".to_string())
    }
}

fn binary_operator(
    operator: impl Fn(i32, i32) -> i32,
    operator_name: &str,
    operand_stack: &mut Vec<i32>,
) -> Result<(), String> {
    if let [.., first, second] = operand_stack[..] {
        operand_stack.truncate(operand_stack.len() - 2);
        operand_stack.push(operator(first, second));
        Ok(())
    } else {
        Err(format!(
            "The Operator {} expects 2 arguments, but received only {}!",
            operator_name,
            operand_stack.len()
        ))
    }
}
