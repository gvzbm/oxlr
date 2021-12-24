//! This module defines the structure of the IR virtual machine code, which is in single static
//! assignment form. Code is always found inside function bodies as [`FnBody`], organized into
//! single [`BasicBlock`]s, each of which represents a continuous path of execution.
use std::borrow::Cow;

use serde::{Serialize, Deserialize};
use super::{Symbol, Path, Type, numbers::Integer, numbers::Float};

/// A reference to a virtual machine register
/// In keeping with SSA form, a register can only be assigned to once in the program, but the value can be used many times
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Register(pub u32);

/// A reference to a [`BasicBlock`] within a function body, by index
pub type BlockIndex = usize;

/// A value that is stored in the IR code
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    /// The literal unit value `()`
    LiteralUnit,
    /// A literal integer value of arbitrary width and sign
    LiteralInt(Integer),
    /// A literal floating point value
    LiteralFloat(Float),
    /// A string literal
    LiteralString(String),
    /// A boolean literal
    LiteralBool(bool),
    /// A reference to the value stored in a register
    Reg(Register)
}

/// Operations on two values that can be executed by the [`BinaryOp`](Instruction::BinaryOp) instruction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Shl, Shr,
    LAnd, LOr, Eq, NEq, Less, Greater, LessEq, GreaterEq
}

/// Operations on a single value that can be executed by the [`UnaryOp`](Instruction::UnaryOp) instruction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UnaryOp {
    LogNot, BitNot, Neg
}


/// A single virtual machine instruction.
/// Destination registers are typically first in the tuple, then the source value
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Instruction {
    /// A SSA Phi node, which selects a value to store in its destination register depending on
    /// which basic block jumped to this instruction
    Phi(
        /// Destination register
        Register,
        /// A mapping from origin blocks to values
        std::collections::HashMap<BlockIndex, Value>
    ),

    /// Jump to a different block based on the value of `cond`
    Br {
        cond: Value,
        /// Index of the block to jump to if `cond` is true
        if_true: BlockIndex,
        /// Index of the block to jump to if `cond` is false
        if_false: BlockIndex
    },

    /// Compute a binary operation from [`BinOp`] on two values, putting the result in the
    /// destination register
    BinaryOp(
        /// Operation to perform
        BinOp,
        /// Destination register
        Register,
        /// First input value
        Value,
        /// Second input value
        Value
    ),
    /// Compute a unary operation from [`UnaryOp`] on a values, putting the result in the
    /// destination register
    UnaryOp(
        /// Operation to perform
        UnaryOp,
        /// Destination register
        Register,
        /// Input value
        Value
    ),

    /// Copy a value into a register directly
    LoadImm(Register, Value),

    /// Get the value behind a reference on the heap
    LoadRef(
        /// Destination register
        Register,
        /// Register containing the source reference
        Register
    ),
    /// Move a value into a reference on the heap
    StoreRef(
        /// Register containing the destination reference
        Register,
        /// Value to store behind reference
        Value
    ),

    /// Compute the reference to a index into a reference to an array or tuple
    RefIndex(
        /// Destination register for reference to inner data
        Register,
        /// Source container reference
        Register,
        /// Index
        Value
    ),

    /// Compute the reference to a field within a structure
    RefField(
        /// Destination register for reference to inner field
        Register,
        /// Source structure reference
        Register,
        /// Field name
        Symbol
    ),

    /// Loads the indexed value starting from zero in the referenced array or tuple on the heap
    LoadIndex(
        /// Destination register
        Register,
        /// Source array/tuple reference
        Register,
        /// Index
        Value
    ),

    /// Stores a value at an index into an array or tuple on the heap
    StoreIndex(
        /// Source reference
        Register,
        /// Index
        Value,
        /// Value to store behind reference
        Value
    ),

    /// Load a value in a field in a structure referenced on the heap
    LoadField(
        /// Destination register
        Register,
        /// Source reference
        Register,
        /// Field name
        Symbol
    ),
    /// Store a value in a field in a structure referenced on the heap
    StoreField(
        /// Value to store in field
        Value,
        /// Register that contains the reference to the destination
        Register,
        /// Field name
        Symbol
    ),

    /// Call a function referenced by the path, placing the return value in the destination register
    Call(
        /// Destination for return value
        Register,
        /// Path to the function
        Path,
        /// Argument values
        Vec<Value>
    ),
    /// Call the implementation function for the specified interface function, placing the return
    /// value in the destination register. The first parameter's type will be used to find the specific implementation
    CallImpl(
        /// Destination for return value
        Register,
        /// Path to the function on the interface (not the implementation)
        Path,
        /// Argument values
        Vec<Value>
    ),
    /// Return from this function, yielding specified value
    Return(Value),

    /// Create a function pointer to a function at the path
    RefFunc(Register, Path),

    /// Test to see if a sum type value matches a specific variant, optionally unwrapping its contained value and putting it in a register
    UnwrapVariant(
        /// Destination register set to true or false depending on if this was a successful match 
        Register,
        /// Optional destination register set to inner value of the variant
        Option<Register>,
        /// The variant value to test
        Value,
        /// The name of the variant to test for
        Symbol
    ),

    /// Allocate a value on the heap of a specified type and put a reference in the destination register
    Alloc(Register, Type),
    /// Allocate an array of values on the heap and put an array value reference in the destination register.
    AllocArray(
        /// Destination register
        Register,
        /// Element type
        Type,
        /// Number of elements in the array
        Value
    )
}

/// A basic block of continuous execution within a function body. Execution proceeds sequentially from the first instruction in `instrs`.
/// If execution comes to the end of the block, the block indexed by `next_block` will be executed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BasicBlock {
    pub instrs: Vec<Instruction>,
    pub next_block: BlockIndex
}

/// The actual code for a function that will be executed when it is called
/// Execution begins at block number 0
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FnBody {
    /// The maximum number of registers that will be used by the function body
    pub max_registers: u32,
    /// The basic blocks that are contained in the function body. [`BlockIndex`] values are indices inside this vector.
    pub blocks: Vec<BasicBlock>
}
