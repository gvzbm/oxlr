use std::borrow::Cow;

use serde::{Serialize, Deserialize};
use super::{Symbol, Path, Type, numbers::Integer, numbers::Float};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Register(pub u32);

pub type BlockIndex = usize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    LiteralUnit,
    LiteralInt(Integer),
    LiteralFloat(Float),
    LiteralString(String),
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
pub enum Instruction {
    Phi(Register, std::collections::HashMap<BlockIndex, Value>),
    Br { cond: Value, if_true: BlockIndex, if_false: BlockIndex },
    BinaryOp(BinOp, Register, Value, Value),
    UnaryOp(UnaryOp, Register, Value),
    LoadImm(Register, Value),

    /// Get the value behind a reference
    /// (Register to put value in, Reference to load from)
    LoadRef(Register, Register),
    /// Move a value into a reference
    /// (Reference to store into, Value to put in)
    StoreRef(Register, Value),
    /// Load at an index into an array or tuple
    LoadIndex(Register, Register, Value),
    /// Store at an index into an array or tuple
    StoreIndex(Register, Register, Value),

    /// Load a value in a field in a structure
    LoadField(Register, Register, Symbol),
    /// Store a value in a field in a structure
    StoreField(Register, Register, Symbol),

    /// (path to function, parameters)
    Call(Register, Path, Vec<Value>),
    /// (path to interface function, parameters) the first parameter's type will be used to find the implementation
    CallImpl(Register, Path, Vec<Value>),
    Return(Value),

    /// create a function pointer
    RefFunc(Register, Path),
    /// (destination for true/false if matched, destination for inner value, value to test, variant to test for)
    UnwrapVariant(Register, Option<Register>, Value, Symbol),
    /// allocate a value on the heap and put a reference in the destination register
    Alloc(Register, Type),
    /// allocate an array of values on the heap and put an array value in the destination register
    AllocArray(Register, Type, Value)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BasicBlock {
    pub instrs: Vec<Instruction>,
    pub next_block: BlockIndex
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FnBody {
    pub max_registers: u32,
    pub blocks: Vec<BasicBlock>
}
