use std::{cell::RefCell, collections::HashMap, rc::Rc, borrow::Cow};
use itertools::Itertools;
use anyhow::*;

mod world;
mod value;
mod memory;

use world::World;
use value::*;
use memory::{Memory, Frame};

unsafe fn memcpy(src: *const u8, dest: *mut u8, size: usize) {
    let src_data = std::slice::from_raw_parts(src, size);
    let dest_data = std::slice::from_raw_parts_mut(dest, size);
    for (s, d) in src_data.iter().zip(dest_data.iter_mut()) {
        *d = *s;
    }
}

struct Machine<'w> {
    world: &'w World,
    mem: Memory<'w>,
}

impl<'w> Machine<'w> {
    fn new(world: &'w World) -> Machine {
        Machine {
            mem: Memory::new(world), world
        }
    }

    /// start the virtual machine
    fn start(&mut self, mut starting_module_path: ir::Path) {
        starting_module_path.0.push(ir::Symbol("start".into()));
        let (_, body) = self.world.get_function(&starting_module_path)
            .expect("a start function is present");
        log::trace!("starting execution");
        let rv = self.call_fn(body, vec![]).unwrap();
        println!("{} returned: {:?}", starting_module_path, rv);
    }

    /// look up and call a function by interpreting its body to determine the return value
    fn call_fn(&mut self, body: &'w ir::FnBody, args: Vec<Value>) -> Result<Value> {
        self.mem.stack.push(Frame::new(body.max_registers as usize));
        for (i, v) in args.into_iter().enumerate() {
            self.mem.cur_frame().store(&ir::code::Register(i as u32), v);
        }
        let mut cur_block_index = 0;
        let mut prev_block_index: Option<usize> = Some(0);
        'blocks: loop {
            let cur_block = &body.blocks[cur_block_index];
            for instr in cur_block.instrs.iter() {
                log::debug!("running instruction {:?}", instr);
                log::debug!("current frame {:?}", self.mem.cur_frame());
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
                            (BinOp::Sub, Value::Int(a), Value::Int(b)) => {
                                // do a saturating subtraction for now
                                // TODO: deal with overflow
                                if a.data < b.data {
                                    Value::Int(Integer::new(a.width, a.signed, 0))
                                } else {
                                    Value::Int(a-b)
                                }
                            },
                            (BinOp::Mul, Value::Int(a), Value::Int(b)) => Value::Int(a*b),
                            (BinOp::Div, Value::Int(a), Value::Int(b)) => Value::Int(a/b),
                            (BinOp::Eq,  a, b) => Value::Bool(a == b),
                            (BinOp::NEq,  a, b) => Value::Bool(a != b),
                            //TODO: implement the rest of the binary operators. for most of these,
                            //the operation also needs to be added to the corrosponding value as
                            //well (Integer/Float). Additionally, invalid/mismatched types should
                            //result in an actual error rather than panicking.
                            (op, lhs, rhs) => todo!("unimplemented binary operator {:?} ({:?}) {:?}", lhs, op, rhs)
                        };
                        self.mem.cur_frame().store(dest, res);
                    },
                    Instruction::UnaryOp(op, dest, inp) => {
                        use ir::code::UnaryOp;
                        let inp = self.mem.cur_frame().convert_value(inp);
                        let res = match (op, inp) {
                            (UnaryOp::LogNot, Value::Bool(v)) => Value::Bool(!v),
                            (UnaryOp::BitNot, Value::Int(v)) => Value::Int(v.bitwise_negate()),
                            (UnaryOp::Neg,    Value::Int(v)) if v.signed => Value::Int(v.negate()),
                            _ => bail!("invalid operand to unary operation")
                        };
                        self.mem.cur_frame().store(dest, res);
                    },

                    Instruction::LoadImm(dest, v) => {
                        let v = self.mem.cur_frame().convert_value(v);
                        self.mem.cur_frame().store(dest, v)
                    },
                    Instruction::LoadRef(dest, r#ref) => {
                        match self.mem.cur_frame().load(r#ref) {
                            Value::Ref(r) => self.mem.cur_frame().store(dest, r.value()),
                            v => bail!("expected ref, got: {:?}", v)
                        }
                    },
                    Instruction::StoreRef(dest, src) => {
                        match self.mem.cur_frame().load(dest) {
                            Value::Ref(r) => r.set_value(self.mem.cur_frame().convert_value(src)),
                            v => bail!("expected ref, got: {:?}", v)
                        }
                    },

                    Instruction::RefField(dest, src_ref, field) => {
                        match self.mem.cur_frame().load(src_ref) {
                            Value::Ref(r) => self.mem.cur_frame().store(dest,
                                Value::Ref(r.field(self.world, field)?)),
                            _ => bail!("expected ref")
                        }
                    }
                    Instruction::LoadField(dest, r#ref, field) => {
                        match self.mem.cur_frame().load(r#ref) {
                            Value::Ref(r) => self.mem.cur_frame().store(dest,
                                r.field(self.world, field)?.value()),
                            _ => bail!("expected ref")
                        }
                    },
                    Instruction::StoreField(src, r#ref, field) => {
                        match self.mem.cur_frame().load(r#ref) {
                            Value::Ref(r) => {
                                let val = self.mem.cur_frame().convert_value(src);
                                r.field(self.world, field)?.set_value(val)
                            },
                            _ => bail!("expected ref")
                        }
                    },

                    Instruction::RefIndex(dest, src_ref, index) => {
                        let index = match self.mem.cur_frame().convert_value(index) {
                            Value::Int(Integer { signed: false, data, .. }) => data as usize,
                            _ => bail!("invalid index")
                        };
                        match self.mem.cur_frame().load(src_ref) {
                            Value::Ref(r) =>
                                self.mem.cur_frame().store(dest,
                                    Value::Ref(r.indexed(self.world, index)?)),
                            _ => bail!("expected ref or array")
                        }
                    },
                    Instruction::LoadIndex(dest, r#ref, index) => {
                        let index = match self.mem.cur_frame().convert_value(index) {
                            Value::Int(Integer { signed: false, data, .. }) => data as usize,
                            _ => bail!("invalid index")
                        };
                        match self.mem.cur_frame().load(r#ref) {
                            Value::Ref(r) =>
                                self.mem.cur_frame().store(dest,
                                    r.indexed(self.world, index)?.value()),
                            _ => bail!("expected ref or array")
                        }
                    },
                    Instruction::StoreIndex(r#ref, index, src) => {
                        let index = match self.mem.cur_frame().convert_value(index) {
                            Value::Int(Integer { signed: false, data, .. }) => data as usize,
                            _ => bail!("invalid index")
                        };
                        match self.mem.cur_frame().load(r#ref) {
                            Value::Ref(r) => {
                                let val = self.mem.cur_frame().convert_value(src);
                                r.indexed(self.world, index)?.set_value(val);
                            },
                            _ => bail!("expected ref or array")
                        }
                    }

                    Instruction::Call(dest, fn_path, params) => {
                        log::trace!("calling {}", fn_path);
                        // TODO: Check types to make sure call is valid!
                        let (fn_sig, fn_body) = self.world.get_function(fn_path).ok_or_else(|| anyhow!("function not found"))?;
                        let params = params.iter().map(|p| self.mem.cur_frame().convert_value(p)).collect();
                        let result = self.call_fn(fn_body, params)?;
                        self.mem.cur_frame().store(dest, result)
                    },
                    Instruction::CallImpl(dest, fn_path, params) => {
                        log::trace!("calling {}", fn_path);
                        // TODO: Check types to make sure call is valid!
                        let params: Vec<Value> = params.iter().map(|p| self.mem.cur_frame().convert_value(p)).collect();
                        let self_val = params.first().ok_or_else(|| anyhow!("call impl requires at least one parameter"))?;
                        let (fn_sig, fn_body) = self.world.find_impl(fn_path, &self_val.type_of(&self.mem))
                            .ok_or_else(|| anyhow!("implementation not found"))?;
                        let result = self.call_fn(fn_body, params)?;
                        self.mem.cur_frame().store(dest, result)
                    },
                    Instruction::Return(v) => {
                        log::trace!("return");
                        let rv = self.mem.cur_frame().convert_value(v);
                        self.mem.pop_stack();
                        return Ok(rv)
                    },
                    Instruction::RefFunc(dest, _) => todo!(),
                    Instruction::UnwrapVariant(dest, _, _, _) => todo!(),
                    Instruction::Alloc(dest, r#type) => {
                        let nrf = self.mem.alloc(r#type)?;
                        self.mem.cur_frame().store(dest, nrf);
                    },
                    Instruction::AllocArray(dest, r#type, count) => {
                        let count = match self.mem.cur_frame().convert_value(count) {
                            Value::Int(Integer { signed: false, data, .. }) => data as usize,
                            _ => bail!("invalid count for array alloc")
                        };
                        let nrf = self.mem.alloc_array(r#type, count)?;
                        self.mem.cur_frame().store(dest, nrf);
                    },
                    Instruction::StackAlloc(dest, r#type) => {
                        let nrf = self.mem.stack_alloc(r#type)?;
                        self.mem.cur_frame().store(dest, nrf);
                    },
                    Instruction::StackAllocArray(dest, r#type, count) => {
                        let count = match self.mem.cur_frame().convert_value(count) {
                            Value::Int(Integer { signed: false, data, .. }) => data as usize,
                            _ => bail!("invalid count for array alloc")
                        };
                        let nrf = self.mem.stack_alloc_array(r#type, count)?;
                        self.mem.cur_frame().store(dest, nrf);
                    },

                    Instruction::CopyToStack(dest, src) => {
                        match self.mem.cur_frame().load(src) {
                            Value::Ref(memory::Ref { ty, data }) => {
                                let (copy, size) = if let ir::Type::Array(el_ty) = ty.as_ref() {
                                    let count = unsafe { *(data as *mut usize) };
                                    (self.mem.stack_alloc_array(el_ty.as_ref(), count)?,
                                        self.world.array_size(el_ty, count)?)
                                } else {
                                    (self.mem.stack_alloc(ty.as_ref())?,
                                        self.world.size_of_type(ty.as_ref())?)
                                };
                                if let Value::Ref(copy) = &copy {
                                    unsafe { memcpy(data, copy.data, size); }
                                } else { unreachable!() }
                                self.mem.cur_frame().store(dest, copy);
                            }
                            _ => bail!("expected ref")
                        }
                    },

                    // sad code duplication - should there just be a single alloc function with a
                    // destination argument instead?
                    Instruction::CopyToHeap(dest, src) => {
                        match self.mem.cur_frame().load(src) {
                            Value::Ref(memory::Ref { ty, data }) => {
                                let (copy, size) = if let ir::Type::Array(el_ty) = ty.as_ref() {
                                    let count = unsafe { *(data as *mut usize) };
                                    (self.mem.alloc_array(el_ty.as_ref(), count)?,
                                        self.world.array_size(el_ty, count)?)
                                } else {
                                    (self.mem.alloc(ty.as_ref())?,
                                        self.world.size_of_type(ty.as_ref())?)
                                };
                                if let Value::Ref(copy) = &copy {
                                    unsafe { memcpy(data, copy.data, size); }
                                } else { unreachable!() }
                                self.mem.cur_frame().store(dest, copy);
                            }
                            _ => bail!("expected ref")
                        }
                    }
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
    m.start(start_mod_path);
}
