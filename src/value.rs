use std::{fmt::Display, rc::Rc};

// #[derive(Debug, Clone)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
    // TODO: string interning? may not be necessary
    String(Rc<String>),
}

#[macro_export]
macro_rules! binary_arithmetic_op {
    ($v1:ident $op:tt $v2:ident) => {
        match ($v1, $v2) {
            (Value::Number(n1), Value::Number(n2)) => {
                let n1 = <f64>::clone(&n1);
                let n2 = <f64>::clone(&n2);
                Ok(Value::Number(n1 $op n2))
            }
            _ => Err("values must both be either strings or numbers"),
        }
    };
}

#[macro_export]
macro_rules! binary_boolean_op {
    ($v1:ident $op:tt $v2:ident) => {
        match ($v1, $v2) {
            (Value::Number(n1), Value::Number(n2)) => {
                let n1 = <f64>::clone(&n1);
                let n2 = <f64>::clone(&n2);
                Ok(Value::Boolean(n1 $op n2))
            }
            _ => Err("values must both be either strings or numbers"),
        }
    };
}

impl Value {
    pub fn concatenate_strings(v1: &Value, v2: &Value) -> Result<Value, &'static str> {
        match (v1, v2) {
            (Value::String(s1), Value::String(s2)) => {
                let mut s1 = String::clone(s1);
                let s2 = String::clone(s2);
                s1.push_str(&s2);
                return Ok(Value::String(Rc::new(s1)));
            }
            _ => Err("values must both be either strings or numbers"),
        }
    }

    // TODO implement PartialEq for Value instead
    pub fn equals(v1: Value, v2: Value) -> bool {
        match v1 {
            Value::Boolean(b1) => match v2 {
                Value::Boolean(b2) => b1 == b2,
                _ => false,
            },
            Value::Number(n1) => match v2 {
                Value::Number(n2) => n1 == n2,
                _ => false,
            },
            Value::Nil => match v2 {
                Value::Nil => true,
                _ => false,
            },
            Value::String(s1) => match v2 {
                Value::String(s2) => s1.eq(&s2),
                _ => false,
            },
        }
    }

    pub fn is_string(v: &Value) -> bool {
        if let Value::String(_) = v {
            true
        } else {
            false
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Nil => write!(f, "{}", "NIL"),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}
