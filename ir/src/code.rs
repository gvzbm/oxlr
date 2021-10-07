use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction { }

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicBlock {
    pub instrs: Vec<Instruction>,
    pub next_block: usize
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FnBody {
    pub blocks: Vec<BasicBlock>
}
