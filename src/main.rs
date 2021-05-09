mod chunk;
// mod value;

use chunk::*;

fn main() {
    // println!("A");
    // println!("{} bytes", std::mem::size_of::<OpCode>());
    // println!("{} bytes", std::mem::size_of::<Vec<OpCode>>());
    // println!("{} bytes", std::mem::size_of::<Vec<u8>>());

    let mut chunk = Chunk::init();
    chunk.write(OpCode::OpReturn);
    chunk.disassemble("test chunk");
}
