use crate::world::*;
use crate::value::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc, borrow::Cow, alloc::Layout, mem::size_of, ptr::null_mut};
use anyhow::*;
use itertools::Itertools;

struct Header {
    ty: Box<ir::Type>, // this could probably be a &'m instead
    elements: usize,
    prev: *mut Header
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeapRef(*mut u8);

pub struct Memory<'w> {
    world: &'w World,
    last_alloc: *mut Header,
    max_size: usize, current_size: usize,

    pub stack: Vec<Frame>
}

impl HeapRef {
    pub fn type_of(&self) -> &ir::Type {
        unsafe { &(*(self.0 as *mut Header)).ty }
    }

    pub fn element_count(&self) -> usize {
        unsafe { (*(self.0 as *mut Header)).elements }
    }

    fn value_at_offset(&self, offset_in_bytes: usize, ty: &ir::Type) -> Value {
        unsafe {
            let ptr = self.0.offset((std::mem::size_of::<Header>() + offset_in_bytes) as isize);
            match ty {
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
                ir::Type::Array(_) => Value::Array(HeapRef(*(ptr as *mut *mut u8))),
                ir::Type::Ref(_) | ir::Type::AbstractRef(_) 
                    => Value::Ref(HeapRef(*(ptr as *mut *mut u8))),
                _ => todo!()
            }
        }
    }

    // `ty` is the type of the thing that is already there
    fn set_value_at_offset(&self, offset_in_bytes: usize, ty: &ir::Type, val: Value) {
        unsafe {
            let ptr = self.0.offset((std::mem::size_of::<Header>() + offset_in_bytes) as isize);
            match (ty, val) {
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
                (ir::Type::Array(_), Value::Array(r)) => {
                    // should we validate the element type here?
                    *(ptr as *mut *mut u8) = r.0;
                }
                (ir::Type::Ref(_) | ir::Type::AbstractRef(_), Value::Ref(r)) => {
                    // should we validate the type here?
                    *(ptr as *mut *mut u8) = r.0;
                }
                _ => todo!()
            }
        }
    }


    pub fn value(&self) -> Value { self.value_at_offset(0, self.type_of()) }
    pub fn set_value(&self, val: Value) { self.set_value_at_offset(0, self.type_of(), val) }

    fn indexed_offset(&self, world: &World, index: usize) -> Result<(usize, &ir::Type)> {
        match self.type_of() {
            ir::Type::Array(el_ty) => Ok((index * world.size_of_type(el_ty)?, el_ty)) ,
            ir::Type::Tuple(ts) => {
                let mut offset = 0;
                for t in ts.iter().take(index) {
                    let ralign = world.required_alignment(t)?;
                    while offset % ralign != 0 { offset += 1; }
                    offset += world.size_of_type(t)?;
                }
                Ok((offset, &ts[index]))
            },
            _ => Err(anyhow!("cannot index into unindexed type"))
        }
    }

    pub fn indexed_value(&self, world: &World, index: usize) -> Result<Value> {
        let (offset, ty) = self.indexed_offset(world, index)?;
        Ok(self.value_at_offset(offset, ty))
    }

    pub fn set_indexed_value(&self, world: &World, index: usize, val: Value) -> Result<()> {
        let (offset, ty) = self.indexed_offset(world, index)?;
        Ok(self.set_value_at_offset(offset, ty, val))
    }

    fn field_offset<'w>(&self, world: &'w World, field: &ir::Symbol) -> Result<(usize, &'w ir::Type)> {
        match self.type_of() {
            ir::Type::User(path, None) => {
                match world.get_type(path) {
                    Some(ir::TypeDefinition::NewType(ty)) => {
                        Ok((0, ty)) // should probably check what the field name is?
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
                            Ok((offset, ty))
                        } else {
                            Err(anyhow!("field not defined on type"))
                        }
                    },
                    None => Err(anyhow!("unknown type")),
                }
            },
            ir::Type::User(path, Some(params)) => todo!(),
            _ => Err(anyhow!("invalid type for field lookup"))
        }
    }

    pub fn field_value(&self, world: &World, field: &ir::Symbol) -> Result<Value> {
        let (offset, ty) = self.field_offset(world, field)?;
        Ok(self.value_at_offset(offset, ty))
    }

    pub fn set_field_value(&self, world: &World, field: &ir::Symbol, val: Value) -> Result<()> {
        let (offset, ty) = self.field_offset(world, field)?;
        Ok(self.set_value_at_offset(offset, ty, val))
    }
}

//TODO: it is most likely going to be the most convenient to control the layout of memory in the
//heap. need to transition from using *mut Value in refs to having some sort of heap handle type
//thing that allows for reading/writing the value in the heap and automatically takes care of field
//accesses/array indexing. It should be easy to get the right info from the heap metadata and then
//calculate the byte offset and convert the bytes into a value

impl<'w> Memory<'w> {
    pub fn new(world: &'w World) -> Memory {
        Memory {
            world, last_alloc: null_mut(),
            max_size: 4 * 1024 * 1024 * 1024, // 4GiB
            current_size: 0,
            stack: Vec::new()
        }
    }

    /// allocate a new value on the heap, and return a reference value
    pub fn alloc(&mut self, ty: &ir::Type) -> Result<Value> {
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
                (&mut *mem).ty = Box::new(ty.clone());
                (&mut *mem).elements = 1;
                // make sure we can still find this allocation if there aren't any other references to
                // it when we do garbage collection
                (&mut *mem).prev = self.last_alloc;
                self.last_alloc = mem;
                self.current_size += layout.size();
                // should this be aligned? how do we know how much padding to allocate until after
                // we get the pointer?
                return Ok(Value::Ref(HeapRef(mem.offset(1) as *mut u8)));
            }
        }
    }

    pub fn alloc_array(&mut self, el_ty: &ir::Type, count: usize) -> Result<Value> {
        let mut ran_gc = false;
        loop {
            let layout = Layout::from_size_align(size_of::<Header>() + self.world.size_of_type(el_ty)?*count, 
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
                (&mut *mem).ty = Box::new(el_ty.clone());
                (&mut *mem).elements = count;
                // make sure we can still find this allocation if there aren't any other references to
                // it when we do garbage collection
                (&mut *mem).prev = self.last_alloc;
                self.last_alloc = mem;
                self.current_size += layout.size();
                // should this be aligned?
                return Ok(Value::Array(HeapRef(mem.offset(1) as *mut u8)));
            }
        }
    }

    pub fn type_for(&self, rf: *mut Value) -> ir::Type {
        let header = unsafe {&*((rf as *mut u8).offset(-(size_of::<Header>() as isize)) as *mut Header)};
        *(header.ty.clone())
    }

    pub fn element_count(&self, rf: *mut Value) -> usize {
        let header = unsafe {&*((rf as *mut u8).offset(-(size_of::<Header>() as isize)) as *mut Header)};
        header.elements
    }

    /// move a value into the heap, returning a reference value
    pub fn box_value(&mut self, val: Value) -> Result<Value> {
        match self.alloc(&val.type_of(self))? {
            Value::Ref(r) => {
                r.set_value(val);
                Ok(Value::Ref(r))
            }
            _ => unreachable!()
        }
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
    pub registers: Vec<Value>
}

impl Frame {
    pub fn new(num_reg: usize) -> Frame {
        Frame {
            registers: std::iter::repeat(Value::Nil).take(num_reg).collect()
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
            ir::code::Value::LiteralInt(d) => Value::Int(Integer::unsigned(64, *d as u64)),
            ir::code::Value::LiteralFloat(_) => todo!(),
            ir::code::Value::LiteralString(_) => todo!(),
            ir::code::Value::Reg(r) => self.registers[r.0 as usize].clone(),
        }
    }
}


