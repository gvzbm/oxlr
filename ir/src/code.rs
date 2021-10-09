use serde::{Serialize, Deserialize};
use super::{Symbol, Path, Type};

#[derive(Serialize, Deserialize, Debug)]
pub struct Register(u32);

pub type BlockIndex = usize;

#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
    Literal(usize),
    Reg(Register)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Shl, Shr,
    LAnd, LOr, Eq, NEq, Less, Greater, LessEq, GreaterEq
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UnaryOp {
    LogNot, BitNot, Neg
}


#[derive(Serialize, Deserialize, Debug)]
pub enum Instruction {
    Phi(Vec<(Value, BlockIndex)>),
    Br { cond: Value, if_true: BlockIndex, if_false: BlockIndex },
    BinaryOp(BinOp, Register, Value, Value),
    UnaryOp(UnaryOp, Register, Value, Value),
    Store(Register, Value),
    LoadRef(Register, Register),
    StoreRef(Register, Register),
    LoadAt(Register, Register, usize), // for strings, arrays, tuples and structs
    StoreAt(Register, Register, usize),
    // (path to function, parameters)
    Call(Path, Vec<Value>),
    // (path to interface function, first parameter [type will be used to find implementation], rest of parameters)
    CallImpl(Path, Value, Vec<Value>),
    Return(Option<Value>),
    /// create a function pointer
    RefFunc(Register, Path),
    /// (destination for true/false if matched, destination for inner value, value to test, variant to test for)
    UnwrapVariant(Register, Option<Register>, Value, Symbol)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicBlock {
    pub instrs: Vec<Instruction>,
    pub next_block: BlockIndex
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FnBody {
    pub blocks: Vec<BasicBlock>
}
