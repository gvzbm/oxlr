use std::{cell::RefCell, collections::HashMap, rc::Rc, borrow::Cow};
use itertools::Itertools;
use anyhow::*;

mod world;
mod value;
mod memory;

use world::World;
use value::*;
use memory::{Memory, Frame};

struct Machine<'w> {
    world: &'w World<'w>,
    mem: Memory<'w>,
}

impl<'w> Machine<'w> {
    fn new(world: &'w World<'w>) -> Machine<'w> {
        Machine {
            mem: Memory::new(world), world
        }
    }

    /// start the virtual machine
    fn start(&mut self) {
    //     let start_sym = &"start".into();
    //     let (_, body) = self.world.get_function(&start_sym)
    //         .expect("a start function is present");
    //     let _ = self.call_fn(body, vec![]).unwrap();
    }

    /// look up and call a function by interpreting its body to determine the return value
    fn call_fn(&mut self, body: &'w ir::FnBody<'w>, args: Vec<Value>) -> Result<Value> {
        self.mem.stack.push(Frame::new(body.max_registers as usize));
        // let cur_frame = self.mem.stack.last_mut().unwrap();
        let mut cur_block_index = 0;
        let mut prev_block_index: Option<usize> = Some(0);
        'blocks: loop {
            let cur_block = &body.blocks[cur_block_index];
            for instr in cur_block.instrs.iter() {
                use ir::code::Instruction;
                match instr {
                    Instruction::Phi(dest, precedents) => {
                        let res = self.mem.cur_frame().convert_value(&precedents[prev_block_index.as_ref().unwrap()]);
                        self.mem.cur_frame().store(dest, res)
                    },
                    Instruction::Br { cond, if_true, if_false } => {
                        // ostensibly this is the last instruction in the block
                        match self.mem.cur_frame().convert_value(cond) {
                            Value::Bool(true) => {
                                prev_block_index = Some(cur_block_index);
                                cur_block_index = *if_true;
                                continue 'blocks;
                            },
                            Value::Bool(false) => {
                                prev_block_index = Some(cur_block_index);
                                cur_block_index = *if_false;
                                continue 'blocks;
                            },
                            _ => bail!("expected bool")
                        }
                    },
                    Instruction::BinaryOp(op, dest, lhs, rhs) => {
                        use ir::code::BinOp;
                        let lhs = self.mem.cur_frame().convert_value(lhs);
                        let rhs = self.mem.cur_frame().convert_value(rhs);
                        let res = match (op, lhs, rhs) {
                            (BinOp::Add, Value::Int(a), Value::Int(b)) => Value::Int(a+b),
                            (BinOp::Sub, Value::Int(a), Value::Int(b)) => Value::Int(a-b),
                            (BinOp::Mul, Value::Int(a), Value::Int(b)) => Value::Int(a*b),
                            (BinOp::Div, Value::Int(a), Value::Int(b)) => Value::Int(a/b),
                            (BinOp::Eq,  a, b) => Value::Bool(a == b),
                            (BinOp::NEq,  a, b) => Value::Bool(a != b),
                            //TODO: implement the rest of the binary operators. for most of these,
                            //the operation also needs to be added to the corrosponding value as
                            //well (Integer/Float). Additionally, invalid/mismatched types should
                            //result in an actual error rather than panicking.
                            _ => todo!()
                        };
                        self.mem.cur_frame().store(dest, res);
                    },
                    Instruction::UnaryOp(op, dest, inp) => {
                        use ir::code::UnaryOp;
                        let inp = self.mem.cur_frame().convert_value(inp);
                        let res = match (op, inp) {
                            (UnaryOp::LogNot, Value::Bool(v)) => Value::Bool(!v),
                            (UnaryOp::BitNot, Value::Int(v)) => Value::Int(v.bitwise_negate()),
                            (UnaryOp::Neg,    Value::Int(v)) if v.signed() => Value::Int(v.negate()),
                            _ => bail!("invalid operand to unary operation")
                        };
                        self.mem.cur_frame().store(dest, res);
                    },
                    Instruction::Store(dest, v) => {
                        let v = self.mem.cur_frame().convert_value(v);
                        self.mem.cur_frame().store(dest, v)
                    },
                    Instruction::LoadRef(dest, r#ref) => {
                        match self.mem.cur_frame().load(r#ref) {
                            Value::Ref(r) => self.mem.cur_frame().store(dest, unsafe { (&*r).clone() }),
                            _ => bail!("expected ref")
                        }
                    },
                    Instruction::StoreRef(dest, src) => {
                        match self.mem.cur_frame().load(dest) {
                            Value::Ref(r) => unsafe { *r = self.mem.cur_frame().load(src) },
                            _ => bail!("expected ref")
                        }
                    },
                    Instruction::RefAt(dest, target, index) => {
                        let index = match self.mem.cur_frame().convert_value(index) {
                            Value::Int(Integer::U8(x)) => x as usize,
                            Value::Int(Integer::U16(x)) => x as usize,
                            Value::Int(Integer::U32(x)) => x as usize,
                            Value::Int(Integer::U64(x)) => x as usize,
                            _ => bail!("invalid index")
                        };
                        let target = match self.mem.cur_frame().load(target) {
                            Value::Ref(r) => r, _ => bail!("expected ref")
                        };
                        let targ_type = self.mem.type_for(target);
                        match targ_type {
                            ir::Type::User(path, _) => {},
                            ir::Type::Tuple(tys) => {},
                            ir::Type::Array(el) => {
                                let max = self.mem.element_count(target);
                                if max < index { bail!("invalid index, greater than length"); }
                                unsafe {
                                    self.mem.cur_frame().store(dest, Value::Ref(target.offset(index as isize)));
                                }
                            },
                            _ => bail!("invalid target")
                        }
                    },
                    Instruction::Call(dest, fn_path, params) => {
                        let (fn_sig, fn_body) = self.world.get_function(fn_path).ok_or_else(|| anyhow!("function not found"))?;
                        let params = params.iter().map(|p| self.mem.cur_frame().convert_value(p)).collect();
                        let result = self.call_fn(fn_body, params)?;
                        self.mem.cur_frame().store(dest, result)
                    },
                    Instruction::CallImpl(dest, fn_path, params) => {
                        let params: Vec<Value> = params.iter().map(|p| self.mem.cur_frame().convert_value(p)).collect();
                        let self_val = params.first().ok_or_else(|| anyhow!("call impl requires at least one parameter"))?;
                        let (fn_sig, fn_body) = self.world.find_impl(fn_path, &self_val.type_of(&self.mem))
                            .ok_or_else(|| anyhow!("implementation not found"))?;
                        let result = self.call_fn(fn_body, params)?;
                        self.mem.cur_frame().store(dest, result)
                    },
                    Instruction::Return(Some(v)) => return Ok(self.mem.cur_frame().convert_value(v)),
                    Instruction::Return(None) => return Ok(Value::Nil), // this might cause troubles
                    Instruction::RefFunc(dest, _) => todo!(),
                    Instruction::UnwrapVariant(dest, _, _, _) => todo!(),
                    Instruction::Alloc(dest, r#type) => {
                        let nrf = self.mem.alloc(r#type)?;
                        self.mem.cur_frame().store(dest, nrf);
                    },
                    Instruction::AllocArray(dest, r#type, count) => {
                        let count = match self.mem.cur_frame().convert_value(count) {
                            Value::Int(Integer::U8(c)) => c as usize,
                            Value::Int(Integer::U16(c)) => c as usize,
                            Value::Int(Integer::U32(c)) => c as usize,
                            Value::Int(Integer::U64(c)) => c as usize,
                            _ => bail!("invalid count for array alloc")
                        };
                        let nrf = self.mem.alloc_array(r#type, count)?;
                        self.mem.cur_frame().store(dest, nrf);
                    },
                }
            }
            prev_block_index = Some(cur_block_index);
            cur_block_index = cur_block.next_block;
        }
    }

}

fn main() {
    env_logger::init();
    let start_mod_path = std::env::args().nth(1).map(ir::Path::from).expect("module path command line argument");
    let start_mod_version = std::env::args().nth(2)
        .map(|vr| ir::VersionReq::parse(&vr).expect("parse starting module version req"))
        .unwrap_or(ir::VersionReq::STAR);
    let mut world = World::new().expect("initialize world");
    world.load_module(&start_mod_path, &start_mod_version).expect("load starting module");
    let mut m = Machine::new(&world);
    m.start();
}
