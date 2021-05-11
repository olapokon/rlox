#[derive(Debug, Clone, Copy)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
    True,
    False,
}

// TODO: Value Display?
