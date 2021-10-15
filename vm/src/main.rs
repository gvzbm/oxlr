use std::{collections::HashMap, sync::Arc};

use anyhow::*;

struct World {
    top_modules: RefCell<HashMap<String, Arc<ir::Module>>>,
}

impl World {
    fn new() -> World {}

    /// get a module, loading it from the filesystem if necessary by searching the module search paths
    fn load_module(&self, path: &ir::Path) -> Result<Arc<ir::Module>> { }

    /// look up a type definition by path
    fn get_type(&self, path: &ir::Path) -> Option<&ir::TypeDefinition> {}

    /// look up an interface by path
    fn get_interface(&self, path: &ir::Path) -> Option<&ir::Interface> {}

    /// look up a function by path
    fn get_function(&self, path: &ir::Path) -> Option<&(ir::FunctionSignature, ir::FnBody)> {}

    /// look up the implementation function specific to type `ty` for the interface function `interface_fn`
    fn find_impl(&self, interface_fn: &ir::Path, ty: &Type) -> Option<&(ir::FunctionSignature, ir::FnBody)> {}
}

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

enum Float {
    F32(f32),
    F64(f64)
}

enum Value<'h> {
    Unit, Bool(bool),
    String(&'h str),
    Int(Integer),
    Float(Float),
    Ref, Array, Fn
}

struct Heap<'w> {
    world: &'w World
}

impl<'w> Heap<'w> {
    fn new(world:&'w World)->Heap<'w> {
        Heap {
            world
        }
    }

    /// allocate a new value on the heap, and return a reference value
    fn alloc(&mut self, ty: ir::Type) -> Result<Value> {}

    /// move a value into the heap, returning a reference value
    fn box(&mut self, val: Value) -> Result<Value> {}

    /// inform the heap that this reference is being invalidated and that the value it points to
    /// may now be garbage
    fn release(&mut self, reference: Value) {}

    /// run garbage collection
    fn gc(&mut self) { }
}

struct Machine<'w> {
    world: &'w World,
    heap: Heap<'w>
}

impl<'w> Machine<'w> {
    fn new(world: &'w World) -> Machine<'w> {
        Machine {
            heap: Heap::new(world), world
        }
    }

    /// start the virtual machine
    fn start(&mut self) {}

    /// look up and call a function
    fn call_fn(&mut self, path: ir::Path, args: Vec<Value>) -> Result<Value> {}
}

fn main() {
    let mut world = World::new();
    let mut m = Machine::new(&world);
    m.start();
}
