use crate::chunk::Chunk;

pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn compile() -> Result<Chunk, ()> {
        let compiler = Compiler::init();
        Ok(compiler.chunk)
    }

    fn init() -> Compiler {
        Compiler {
            chunk: Chunk::init(),
        }
    }
}
