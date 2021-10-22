use std::{cell::RefCell, collections::HashMap, sync::Arc};

use anyhow::*;

struct World<'m> {
    top_modules: RefCell<HashMap<String, Arc<ir::Module<'m>>>>,
}

impl<'m> World<'m> {
    fn new() -> World<'m> {
        todo!()
    }

    /// get a module, loading it from the filesystem if necessary by searching the module search paths
    fn load_module(&self, path: &ir::Path) -> Result<Arc<ir::Module<'m>>> {
        todo!()
    }

    /// look up a type definition by path
    fn get_type(&self, path: &ir::Path) -> Option<&ir::TypeDefinition> {
        todo!()
    }

    /// look up an interface by path
    fn get_interface(&self, path: &ir::Path) -> Option<&ir::Interface> {
        todo!()
    }

    /// look up a function by path
    fn get_function(&self, path: &ir::Path) -> Option<&(ir::FunctionSignature, ir::FnBody)> {
        todo!()
    }

    /// look up the implementation function specific to type `ty` for the interface function `interface_fn`
    fn find_impl(&self, interface_fn: &ir::Path, ty: &ir::Type) -> Option<&(ir::FunctionSignature, ir::FnBody)> {
        todo!()
    }
}

#[derive(Clone, Debug)]
enum Integer {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    S8(i8),
    S16(i16),
    S32(i32),
    S64(i64),
}

#[derive(Clone, Debug)]
enum Float {
    F32(f32),
    F64(f64)
}

/// Values have an implicit lifetime tied to the Heap they were allocated on
#[derive(Clone, Debug)]
enum Value {
    Nil, Bool(bool),
    String(*mut str),
    Int(Integer),
    Float(Float),
    Ref(*mut Value), Array(*mut Value), Fn
}

struct Heap<'w> {
    world: &'w World<'w>
}

impl<'w> Heap<'w> {
    fn new(world:&'w World<'w>) -> Heap<'w> {
        Heap {
            world
        }
    }

    /// allocate a new value on the heap, and return a reference value
    fn alloc(&mut self, ty: ir::Type) -> Result<Value> {
        todo!()
    }

    /// move a value into the heap, returning a reference value
    fn box_value(&mut self, val: Value) -> Result<Value> {
        todo!()
    }

    /// inform the heap that this reference is being invalidated and that the value it points to
    /// may now be garbage
    fn free(&mut self, reference: Value) {
        todo!()
    }
}

struct Frame {
    registers: Vec<Value>
}

impl Frame {
    fn new(num_reg: usize) -> Frame {
        Frame {
            registers: std::iter::repeat(Value::Nil).take(num_reg).collect()
        }
    }
}

struct Machine<'w> {
    world: &'w World<'w>,
    heap: Heap<'w>,
    stack: Vec<Frame>
}

impl<'w> Machine<'w> {
    fn new(world: &'w World<'w>) -> Machine<'w> {
        Machine {
            heap: Heap::new(world), world, stack: Vec::new()
        }
    }

    /// start the virtual machine
    fn start(&mut self) {
        let (_, body) = self.world.get_function(&"start".into())
            .expect("a start function is present");
        let _ = self.call_fn(body, vec![]).unwrap();
    }

    /// look up and call a function by interpreting its body to determine the return value
    fn call_fn(&mut self, body: &ir::FnBody, args: Vec<Value>) -> Result<Value> {
        self.stack.push(Frame::new(body.max_registers as usize));
        let cur_frame = self.stack.last().unwrap();
        todo!()
    }

    /// run garbage collection
    fn gc(&mut self) {
        // gc needs access to both the stack and heap to know what is alive
        todo!()
    }
}

fn main() {
    let world = World::new();
    let mut m = Machine::new(&world);
    m.start();
}
