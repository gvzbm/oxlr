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
pub enum Instruction<'a> {
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
    Call(Register, Path<'a>, Vec<Value>),
    // (path to interface function, first parameter [type will be used to find implementation], rest of parameters)
    CallImpl(Register, Path<'a>, Value, Vec<Value>),
    Return(Option<Value>),
    /// create a function pointer
    RefFunc(Register, Path<'a>),
    /// (destination for true/false if matched, destination for inner value, value to test, variant to test for)
    UnwrapVariant(Register, Option<Register>, Value, Symbol<'a>),
    /// allocate a value on the heap and put a reference in the destination register
    Alloc(Register, Type<'a>),
    /// allocate an array of values on the heap and put an array value in the destination register
    AllocArray(Register, Type<'a>, Value)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicBlock<'a> {
    pub instrs: Vec<Instruction<'a>>,
    pub next_block: BlockIndex
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FnBody<'a> {
    pub max_registers: u32,
    pub blocks: Vec<BasicBlock<'a>>
}
