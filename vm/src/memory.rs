use crate::world::*;
use crate::value::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc, borrow::Cow, alloc::Layout, mem::size_of, ptr::null_mut};
use anyhow::*;

struct Header<'m> {
    ty: Box<ir::Type<'m>>,
    elements: usize,
    prev: *mut Header<'m>
}

pub struct Memory<'w> {
    world: &'w World<'w>,
    last_alloc: *mut Header<'w>,
    max_size: usize, current_size: usize,

    pub stack: Vec<Frame>
}


impl<'w> Memory<'w> {
    pub fn new(world:&'w World<'w>) -> Memory<'w> {
        Memory {
            world, last_alloc: null_mut(),
            max_size: 4 * 1024 * 1024 * 1024, // 4GiB
            current_size: 0,
            stack: Vec::new()
        }
    }

    /// allocate a new value on the heap, and return a reference value
    pub fn alloc(&mut self, ty: &ir::Type<'w>) -> Result<Value> {
        if let ir::Type::Array(_) = ty {
            bail!("use alloc_array to allocate arrays");
        }

        let mut ran_gc = false;
        loop {
            let layout = Layout::from_size_align(size_of::<Header>() + self.world.size_of_type(ty)?, 8)?;
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
                // should this be aligned?
                return Ok(Value::Ref(mem.offset(size_of::<Header>() as isize) as *mut Value));
            }
        }
    }

    pub fn alloc_array(&mut self, el_ty: &ir::Type<'w>, count: usize) -> Result<Value> {
        let mut ran_gc = false;
        loop {
            let layout = Layout::from_size_align(size_of::<Header>() + self.world.size_of_type(el_ty)?*count, 8)?;
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
                return Ok(Value::Array(mem.offset(size_of::<Header>() as isize) as *mut Value));
            }
        }
    }

    pub fn type_for(&self, rf: *mut Value) -> ir::Type<'w> {
        let header = unsafe {&*(rf.offset(-(size_of::<Header>() as isize)) as *mut Header)};
        *(header.ty.clone())
    }

    pub fn element_count(&self, rf: *mut Value) -> usize {
        let header = unsafe {&*(rf.offset(-(size_of::<Header>() as isize)) as *mut Header)};
        header.elements
    }

    /// move a value into the heap, returning a reference value
    pub fn box_value(&mut self, val: Value) -> Result<Value> {
        match self.alloc(&val.type_of(self))? {
            Value::Ref(r) => {
                unsafe { *r = val; }
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
}

pub struct Frame {
    pub registers: Vec<Value>
}

impl Frame {
    pub fn new(num_reg: usize) -> Frame {
        Frame {
            registers: std::iter::repeat(Value::Nil).take(num_reg).collect()
        }
    }
}


