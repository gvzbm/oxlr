use crate::world::*;
use crate::value::*;
use std::{alloc::Layout, mem::size_of, ptr::null_mut};
use anyhow::*;

struct Header<'m> {
    ty: &'m ir::Type,
    elements: usize,
    prev: *mut Header<'m>
}

/// A reference to a value somewhere else
#[derive(Clone, Debug, PartialEq)]
pub struct Ref {
    /// The type of the value behind `data`
    ty: Box<ir::Type>,
    /// A raw pointer to the value data
    data: *mut u8
}

impl Ref {
    pub fn type_of(&self) -> &ir::Type {
        //unsafe { &(*(self.0 as *mut Header)).ty }
        &self.ty
    }

    /// If this reference is to an array, returns the length
    pub fn element_count(&self) -> Option<usize> {
        // unsafe { (*(self.0 as *mut Header)).elements }
        if let ir::Type::Array(_) = self.ty.as_ref() {
            unsafe {
                Some(*(self.data as *mut usize))
            }
        } else {
            None
        }
    }

    /// Read the date inside the ref and return it as a Value
    pub fn value(&self) -> Value {
        unsafe {
            let ptr = self.data;
            match self.ty.as_ref() {
                ir::Type::Unit => Value::Nil,
                ir::Type::Bool => Value::Bool(*ptr > 0),
                ir::Type::Int { signed, width } => {
                    match width {
                        8  => Value::Int(Integer::new(8,  *signed, * ptr as u64)),
                        16 => Value::Int(Integer::new(16, *signed, *(ptr as *mut u16) as u64)),
                        32 => Value::Int(Integer::new(32, *signed, *(ptr as *mut u32) as u64)),
                        64 => Value::Int(Integer::new(64, *signed, *(ptr as *mut u64))),
                        _ => panic!()
                    }
                },
                // TODO: this is quite unsafe, really we should have some way to validate that this
                // is a valid pointer. Perhaps though since this is a private interface it's fine.
                ir::Type::Ref(_) | ir::Type::Array(_) => Value::Ref(Ref {
                    ty: self.ty.clone(),
                    data: *(ptr as *mut *mut u8)
                }),
                ir::Type::AbstractRef(_) => todo!(),
                _ => todo!()
            }
        }
    }

    /// Move the data from the value into the ref
    pub fn set_value(&self, val: Value) {
        unsafe {
            let ptr = self.data;
            match (self.ty.as_ref(), val) {
                (ir::Type::Bool, Value::Bool(b)) => {
                    *ptr = if b { 1 } else { 0 };
                },
                (ir::Type::Int { signed: tsig, width: twid },
                    Value::Int(Integer { signed, width, data })) if signed == *tsig && width <= *twid => {
                    match twid {
                        8 => *ptr = data as u8,
                        16 => *(ptr as *mut u16) = data as u16,
                        32 => *(ptr as *mut u32) = data as u32,
                        64 => *(ptr as *mut u64) = data,
                        _ => panic!()
                    }
                },
                // handle nested references
                (ir::Type::Ref(_), Value::Ref(r)) => {
                    // should we validate the type here?
                    *(ptr as *mut *mut u8) = r.data;
                },
                (ir::Type::Array(_), Value::Ref(r)) => {
                    // should we validate the type here?
                    if let ir::Type::Array(_) = r.type_of() {
                        *(ptr as *mut *mut u8) = r.data;
                    } else {
                        panic!()
                    }
                }
                t => todo!("set value on ref with unimpl inner type {:?}", t)
            }
        }
    }

    pub fn indexed(&self, world: &World, index: usize) -> Result<Ref> {
        //TODO: bounds checking
        match self.type_of() {
            ir::Type::Array(el_ty) => {
                Ok(Ref {
                    data: unsafe {
                        self.data.offset((std::mem::size_of::<usize>() + index * world.size_of_type(el_ty)?) as isize)
                    },
                    ty: el_ty.clone()
                })
            },
            ir::Type::Tuple(ts) => {
                let mut offset = 0;
                for t in ts.iter().take(index) {
                    let ralign = world.required_alignment(t)?; 
                    while offset % ralign != 0 { offset += 1; }
                    offset += world.size_of_type(t)?;
                }
                Ok(Ref {
                    data: unsafe { self.data.offset(offset as isize) },
                    ty: Box::new(ts[index].clone())
                })
            },
            t => Err(anyhow!("cannot index into unindexed type: {:?}", t))
        }
    }

    pub fn field<'w>(&self, world: &'w World, field: &ir::Symbol) -> Result<Ref> {
        match self.type_of() {
            ir::Type::User(path, None) => {
                match world.get_type(path) {
                    Some(ir::TypeDefinition::NewType(ty)) => {
                        Ok(Ref {
                            ty: Box::new(ty.clone()),
                            data: self.data
                        }) // should probably check what the field name is?
                    },
                    Some(ir::TypeDefinition::Sum { .. }) => {
                        Err(anyhow!("invalid type for field lookup"))
                    },
                    Some(ir::TypeDefinition::Product { fields, .. }) => {
                        if let Some((_, ty)) = fields.iter().find(|(n, _)| n == field) {
                            let mut offset = 0;
                            for (_,t) in fields.iter().take_while(|(n,_)| n != field) {
                                let ralign = world.required_alignment(ty)?;
                                while offset % ralign != 0 { offset += 1; }
                                offset += world.size_of_type(t)?;
                            }
                            Ok(Ref {
                                ty: Box::new(ty.clone()),
                                data: unsafe { self.data.offset(offset as isize) }
                            })
                        } else {
                            Err(anyhow!("field not defined on type"))
                        }
                    },
                    None => Err(anyhow!("unknown type, path = {}, field = {:?}", path, field)),
                }
            },
            ir::Type::User(path, Some(params)) => todo!(),
            _ => Err(anyhow!("invalid type for field lookup"))
        }
    }
}

pub struct Memory<'w> {
    world: &'w World,
    last_alloc: *mut Header<'w>,
    max_size: usize, current_size: usize,

    pub stack: Vec<Frame>,
    stack_data: Vec<u8>,
    stack_ptr: usize
}

impl<'w> Memory<'w> {
    pub fn new(world: &'w World) -> Memory {
        let stack_size = 1024 * 1024; // 1 MiB
        let mut stack_data = Vec::with_capacity(stack_size);
        stack_data.resize(stack_size, 0);
        Memory {
            world, last_alloc: null_mut(),
            max_size: 4 * 1024 * 1024 * 1024, // 4GiB
            current_size: 0,
            stack: Vec::new(),
            stack_data,
            stack_ptr: 0
        }
    }

    /// allocate a new value on the heap, and return a reference value
    pub fn alloc(&mut self, ty: &'w ir::Type) -> Result<Value> {
        if let ir::Type::Array(_) = ty {
            bail!("use alloc_array to allocate arrays");
        }

        let mut ran_gc = false;
        loop {
            let layout = Layout::from_size_align(size_of::<Header>() + self.world.size_of_type(ty)?, 
                std::mem::align_of::<Header>())?;
            unsafe {
                if self.current_size + layout.size() > self.max_size {
                    if ran_gc {
                        bail!("memory exhausted, increase max heap size from {} (current size = {}, attempted to allocate {} for {:?})",
                            self.max_size, self.current_size, layout.size(), ty)
                    } else {
                        ran_gc = true;
                        self.gc();
                        continue;
                    }
                }
                // we use the system allocator to get some new memory
                let mem = std::alloc::alloc(layout) as *mut Header;
                (&mut *mem).ty = ty;
                (&mut *mem).elements = 1;
                // make sure we can still find this allocation if there aren't any other references to
                // it when we do garbage collection
                (&mut *mem).prev = self.last_alloc;
                self.last_alloc = mem;
                self.current_size += layout.size();
                // should this be aligned? how do we know how much padding to allocate until after
                // we get the pointer?
                return Ok(Value::Ref(Ref {
                    ty: Box::new(ty.clone()),
                    data: mem.offset(1) as *mut u8
                }));
            }
        }
    }

    /// allocate a new value on the stack, and return a reference value and how much to subtract
    /// from the stack pointer when fininshed with it
    pub fn stack_alloc(&mut self, ty: &'w ir::Type) -> Result<Value> {
        if let ir::Type::Array(_) = ty {
            bail!("use alloc_array to allocate arrays");
        }

        let size = self.world.size_of_type(ty)?;
        if self.stack_ptr + size > self.stack_data.len() {
            bail!("data stack overflow, increase stack size from {} (attempted to allocate {} for {:?})",
            self.stack_data.len(), size, ty)
        }
        // TODO: this isn't aligned
        let mem = self.stack_data[self.stack_ptr..].as_ptr() as *mut u8;
        // Zero out the allocation
        for x in self.stack_data[self.stack_ptr..(self.stack_ptr+size)].iter_mut() {
            *x = 0;
        }
        self.stack_ptr += size;
        self.cur_frame().data_stack_size += size;
        // should this be aligned? how do we know how much padding to allocate until after
        // we get the pointer?
        Ok(Value::Ref(Ref {
            ty: Box::new(ty.clone()),
            data: mem
        }))
    }

    pub fn alloc_array(&mut self, el_ty: &'w ir::Type, count: usize) -> Result<Value> {
        let mut ran_gc = false;
        loop {
            let layout = Layout::from_size_align(
                size_of::<Header>() + size_of::<usize>() + self.world.size_of_type(el_ty)?*count, 
                std::mem::align_of::<Header>())?;
            unsafe {
                if self.current_size + layout.size() > self.max_size {
                    if ran_gc {
                        bail!("memory exhausted, increase max heap size from {} (current size = {}, attempted to allocate {} for {} x {:?})",
                            self.max_size, self.current_size, layout.size(), count, el_ty)
                    } else {
                        ran_gc = true;
                        self.gc();
                        continue;
                    }
                }
                // we use the system allocator to get some new memory
                let mem = std::alloc::alloc(layout) as *mut Header;
                (&mut *mem).ty = el_ty;
                (&mut *mem).elements = count;
                // make sure we can still find this allocation if there aren't any other references to
                // it when we do garbage collection
                (&mut *mem).prev = self.last_alloc;
                let sizep = mem.offset(1);
                *(sizep as *mut usize)  = count;
                self.last_alloc = mem;
                self.current_size += layout.size();
                // should this be aligned?
                return Ok(Value::Ref(Ref {
                    ty: Box::new(ir::Type::Array(Box::new(el_ty.clone()))),
                    data: (sizep as *mut u8)
                }));
            }
        }
    }

    /// allocate a new array on the stack, and return a reference value and how much to subtract
    /// from the stack pointer when fininshed with it
    pub fn stack_alloc_array(&mut self, el_ty: &'w ir::Type, count: usize) -> Result<Value> {
        let size = self.world.size_of_type(el_ty)? * count + size_of::<usize>();
        if self.stack_ptr + size > self.stack_data.len() {
            bail!("data stack overflow, increase stack size from {} (attempted to allocate {} for array {} x {:?})",
            self.stack_data.len(), size, count, el_ty)
        }
        // TODO: this isn't aligned
        let mem = self.stack_data[self.stack_ptr..].as_ptr() as *mut u8;
        // Zero out the allocation
        for x in self.stack_data[self.stack_ptr..(self.stack_ptr+size)].iter_mut() {
            *x = 0;
        }
        unsafe { *(mem as *mut usize) = count; }
        self.stack_ptr += size;
        self.cur_frame().data_stack_size += size;
        // should this be aligned? how do we know how much padding to allocate until after
        // we get the pointer?
        Ok(Value::Ref(Ref {
            ty: Box::new(ir::Type::Array(Box::new(el_ty.clone()))),
            data: mem
        }))
    }

    /// Pop the data and frame stack
    pub fn pop_stack(&mut self) {
        self.stack_ptr -= self.cur_frame().data_stack_size;
        self.stack.pop();
    }

    /// run garbage collection
    pub fn gc(&mut self) {
        // gc needs access to both the stack and heap to know what is alive
        log::info!("running garbage collection. current size={}, max size={}", self.current_size, self.max_size);
    }

    pub fn cur_frame(&mut self) -> &mut Frame {
        self.stack.last_mut().unwrap()
    }
}

#[derive(Debug)]
pub struct Frame {
    pub registers: Vec<Value>,
    pub data_stack_size: usize
}

impl Frame {
    pub fn new(num_reg: usize) -> Frame {
        Frame {
            registers: std::iter::repeat(Value::Nil).take(num_reg).collect(),
            data_stack_size: 0
        }
    }

    pub fn load(&self, ix: &ir::code::Register) -> Value {
        self.registers[ix.0 as usize].clone()
    }

    pub fn store(&mut self, ix: &ir::code::Register, v: Value) {
        self.registers[ix.0 as usize] = v;
    }

    pub fn convert_value(&self, val: &ir::code::Value) -> Value {
        match val {
            ir::code::Value::LiteralUnit => Value::Nil,
            ir::code::Value::LiteralInt(d) => Value::Int(*d),
            ir::code::Value::LiteralFloat(d) => Value::Float(*d),
            ir::code::Value::LiteralString(_) => todo!(),
            ir::code::Value::LiteralBool(b) => Value::Bool(*b),
            ir::code::Value::Reg(r) => self.registers[r.0 as usize].clone(),
        }
    }
}


