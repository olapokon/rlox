use crate::chunk::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: i32,
    pub chunk: Chunk,
    pub name: String,
}
