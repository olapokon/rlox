mod chunk;

use chunk::*;

fn main() {
    let mut chunk = Chunk::init();

    let constant = chunk.add_constant(Value(1.2));
    let constant1 = chunk.add_constant(Value(2.2));
    let constant2 = chunk.add_constant(Value(4.8));

    chunk.write(OpCode::OpConstant, 123);
    chunk.write(OpCode::OpOperand(constant), 123);
    chunk.write(OpCode::OpReturn, 123);
    chunk.write(OpCode::OpReturn, 123);
    chunk.write(OpCode::OpConstant, 123);
    chunk.write(OpCode::OpOperand(constant1), 123);
    chunk.write(OpCode::OpReturn, 123);
    chunk.write(OpCode::OpReturn, 123);
    chunk.write(OpCode::OpReturn, 123);
    chunk.write(OpCode::OpConstant, 123);
    chunk.write(OpCode::OpOperand(constant2), 123);
    chunk.write(OpCode::OpReturn, 123);
    chunk.write(OpCode::OpReturn, 123);


    chunk.disassemble("test chunk");
}
