mod chunk;
mod vm;

use chunk::*;
use vm::*;

fn main() {
    let mut chunk = Chunk::init();

    let constant = chunk.add_constant(Value(1.2));
    let constant1 = chunk.add_constant(Value(2.2));
    let constant2 = chunk.add_constant(Value(4.8));

    chunk.write(OpCode::OpConstant(constant), 1);
    chunk.write(OpCode::OpReturn, 2);
    chunk.write(OpCode::OpReturn, 3);
    chunk.write(OpCode::OpConstant(constant1), 4);
    chunk.write(OpCode::OpReturn, 5);
    chunk.write(OpCode::OpReturn, 6);
    chunk.write(OpCode::OpReturn, 7);
    chunk.write(OpCode::OpConstant(constant2), 8);
    chunk.write(OpCode::OpReturn, 8);
    chunk.write(OpCode::OpReturn, 9);

    chunk.disassemble("test chunk");

    let mut vm = VM::init(&chunk);
    vm.interpret();
}
