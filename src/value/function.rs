use crate::chunk::Chunk;

pub enum FunctionType {
    Function,
    Script,
}

/// The runtime representation of a function.
#[derive(Debug, Clone)]
pub struct Function {
    /// The function' number of parameters.
    pub arity: i32,
    /// The function's chunk of bytecode, to be interpreted by the [VM].
    pub chunk: Chunk,
    /// The function's name.
    pub name: String,
}

impl Function {
    pub fn new() -> Function {
        Function {
            arity: 0,
            name: String::new(),
            chunk: Chunk::new(),
        }
    }
}
