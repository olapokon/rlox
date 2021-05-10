mod chunk;
mod vm;

use chunk::*;
use vm::*;

fn main() {
    let mut chunk = Chunk::init();

    let constant = chunk.add_constant(Value(5.6));
    chunk.write(OpCode::OpConstant(constant), 1);

    let constant = chunk.add_constant(Value(4.4));
    chunk.write(OpCode::OpConstant(constant), 1);

    chunk.write(OpCode::OpAdd, 1);

    let constant = chunk.add_constant(Value(5.0));
    chunk.write(OpCode::OpConstant(constant), 1);

    chunk.write(OpCode::OpDivide, 1);

    // chunk.write(OpCode::OpNegate, 2);
    chunk.write(OpCode::OpReturn, 9);

    chunk.disassemble("test chunk");

    let mut vm = VM::init(&chunk);
    vm.interpret();
}
