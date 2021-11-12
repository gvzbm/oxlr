use std::borrow::Cow;

use serde::{Serialize, Deserialize};
use super::{Symbol, Path, Type};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Register(u32);

pub type BlockIndex = usize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value<'m> {
    LiteralInt(usize),
    LiteralFloat(f64),
    LiteralString(Cow<'m, str>),
    Reg(Register)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Shl, Shr,
    LAnd, LOr, Eq, NEq, Less, Greater, LessEq, GreaterEq
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UnaryOp {
    LogNot, BitNot, Neg
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Instruction<'a> {
    Phi(Vec<(Value<'a>, BlockIndex)>),
    Br { cond: Value<'a>, if_true: BlockIndex, if_false: BlockIndex },
    BinaryOp(BinOp, Register, Value<'a>, Value<'a>),
    UnaryOp(UnaryOp, Register, Value<'a>, Value<'a>),
    Store(Register, Value<'a>),
    LoadRef(Register, Register),
    StoreRef(Register, Register),
    LoadAt(Register, Register, usize), // for strings, arrays, tuples and structs
    StoreAt(Register, Register, usize),
    // (path to function, parameters)
    Call(Register, Path<'a>, Vec<Value<'a>>),
    // (path to interface function, first parameter [type will be used to find implementation], rest of parameters)
    CallImpl(Register, Path<'a>, Value<'a>, Vec<Value<'a>>),
    Return(Option<Value<'a>>),
    /// create a function pointer
    RefFunc(Register, Path<'a>),
    /// (destination for true/false if matched, destination for inner value, value to test, variant to test for)
    UnwrapVariant(Register, Option<Register>, Value<'a>, Symbol<'a>),
    /// allocate a value on the heap and put a reference in the destination register
    Alloc(Register, Type<'a>),
    /// allocate an array of values on the heap and put an array value in the destination register
    AllocArray(Register, Type<'a>, Value<'a>)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BasicBlock<'a> {
    pub instrs: Vec<Instruction<'a>>,
    pub next_block: BlockIndex
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FnBody<'a> {
    pub max_registers: u32,
    pub blocks: Vec<BasicBlock<'a>>
}
