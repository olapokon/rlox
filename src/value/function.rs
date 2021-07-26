use crate::chunk::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: i32,
    pub chunk: Chunk,
    pub name: String,
}

impl Function {
    fn new() -> Function {
        Function {
            arity: 0,
            name : String::new(),
            chunk: Chunk::new(),
        }
    }
}
