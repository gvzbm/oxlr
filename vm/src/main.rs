use std::{cell::RefCell, collections::HashMap, rc::Rc, borrow::Cow};

use anyhow::*;

struct World<'m> {
    global_module_path: std::path::PathBuf,
    local_module_path: std::path::PathBuf,
    modules: HashMap<ir::Path<'m>, Rc<ir::Module<'m>>>,
}

fn test_module_file_candidate(dir_entry: &std::fs::DirEntry, path: &ir::Path, version_req: &ir::VersionReq) -> Option<std::path::PathBuf> {
    let filename = dir_entry.file_name().into_string().ok()?;
    let (fpath, fver) = filename.split_once('_')?;
    let fpath = ir::Path::from(fpath);
    let fver = ir::Version::parse(fver).ok()?;
    if fpath == *path && version_req.matches(&fver) {
        Some(dir_entry.path())
    } else {
        None
    }
}

impl<'m> World<'m> {
    fn new() -> Result<World<'m>> {
        Ok(World {
            global_module_path: std::env::var("OXLR_MODULE_PATH").and_then(PathBuf::try_from)?,
            local_module_path: std::env::current_dir()?,
            modules: HashMap::new()
        })
    }

    /// get a module, loading it from the filesystem if necessary by searching the module search paths
    fn load_module<'s>(&mut self, path: &'s ir::Path<'m>, version: &ir::VersionReq) -> Result<Rc<ir::Module<'m>>> {
        assert!(path.len() > 0);
        if let Some(m) = self.modules.get(path) {
            if version.matches(&m.version) {
                Ok(m.clone())
            } else {
                Err(anyhow!("mismatched versions of module {} required. version loaded: {}, version required: {}", path, m.version, version))
            }
        } else {
            for mod_file in
                std::fs::read_dir(&self.global_module_path)?
                    .filter_map(|re| re.map(|e| test_module_file_candidate(&e, path, version)).transpose())
                    .chain(std::fs::read_dir(&self.local_module_path)?
                                .filter_map(|re| re.map(|e| test_module_file_candidate(&e, path, version)).transpose()))
                    .map(|rfp| rfp.and_then(|fp| Ok((std::fs::File::open(&fp)?, fp))))
            {
                match mod_file {
                    Ok((f, fp)) => {
                        let mp: Result<ir::Module, _> = rmp_serde::from_read(f);
                        match mp {
                            Ok(m) => {
                                if m.path == *path && version.matches(&m.version) {
                                    let m = Rc::new(m);
                                    self.modules.insert(path.clone(), m.clone());
                                    for (import_path, import_version) in m.imports.iter() {
                                        self.load_module(import_path, import_version)?;
                                    }
                                    return Ok(m);
                                }
                            },
                            Err(e) => log::warn!("tried to search for module {} v{}, got error decoding file {}: {}",
                                path, version, fp.display(), e)
                        }
                    },
                    Err(e) => log::error!("tried to search for module {} v{}, got error in process: {}", path, version, e)
                }
            }
            Err(anyhow!("could not find module {} v{}", path, version))
        }
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
    world: &'w mut World<'w>,
    heap: Heap<'w>,
    stack: Vec<Frame>
}

/*
impl<'w> Machine<'w> {
    fn new(world: &'w mut World<'w>) -> Machine<'w> {
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
}*/

fn main() {
    let world = World::new();
//    let mut m = Machine::new(&mut world);
 //   m.start();
}
