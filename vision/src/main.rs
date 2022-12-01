use vm::{chunk::Chunk, op, VM};

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);

    chunk.write(op::CONSTANT, 123);

    chunk.write(constant as u8, 123);

    let constant = chunk.add_constant(3.4);

    chunk.write(op::CONSTANT, 123);

    chunk.write(constant as u8, 123);

    chunk.write(op::ADD, 123);

    let constant = chunk.add_constant(5.6);

    chunk.write(op::CONSTANT, 123);

    chunk.write(constant as u8, 123);

    chunk.write(op::DIVIDE, 123);

    chunk.write(op::NEGATE, 123);

    chunk.write(op::RETURN, 123);

    // println!("{:#?}", chunk);
    // chunk.disassemble("test chunk");

    let mut vm = VM::new();

    vm.interpret(chunk);
}
