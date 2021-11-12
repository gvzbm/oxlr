use std::{cell::RefCell, collections::HashMap, rc::Rc, borrow::Cow};
use itertools::Itertools;

use anyhow::*;

struct World<'m> {
    global_module_path: std::path::PathBuf,
    local_module_path: std::path::PathBuf,
    modules: HashMap<ir::Path<'m>, ir::Module<'m>>,
    instantiated_types: HashMap<(ir::Path<'m>, Vec<ir::Type<'m>>), ir::TypeDefinition<'m>>
}

fn test_module_file_candidate(dir_entry: &std::fs::DirEntry, path: &ir::Path, version_req: &ir::VersionReq) -> Option<std::path::PathBuf> {
    let filename = dir_entry.file_name().into_string().ok()?;
    let (fpath, fver) = filename.split_once('_')?;
    log::trace!("testing {} as candidate for module", dir_entry.path().display());
    let fpath = ir::Path::from(fpath);
    let fver = ir::Version::parse(fver).ok()?;
    log::trace!("candidate yielded: {} {}", fpath, fver);
    if fpath == *path && version_req.matches(&fver) {
        log::trace!("matched");
        Some(dir_entry.path())
    } else {
        None
    }
}

impl<'m> World<'m> {
    fn new() -> Result<World<'m>> {
        Ok(World {
            global_module_path: std::env::var("OXLR_MODULE_PATH").map(std::path::PathBuf::from)?,
            local_module_path: std::env::current_dir()?,
            modules: HashMap::new(),
            instantiated_types: HashMap::new()
        })
    }

    /// get a module, loading it from the filesystem if necessary by searching the module search paths
    fn load_module<'s>(&mut self, path: &'s ir::Path<'m>, version: &ir::VersionReq) -> Result<()> {
        assert!(path.len() > 0);
        if let Some(m) = self.modules.get(path) {
            if version.matches(&m.version) {
                Ok(())
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
                                    for (import_path, import_version) in m.imports.iter() {
                                        self.load_module(import_path, import_version)?;
                                    }
                                    self.modules.insert(path.clone(), m);
                                    return Ok(());
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

    fn get_module(&self, path: &ir::Path<'m>) -> Option<&ir::Module> {
        self.modules.get(path)
    }

    /// look up a type definition by path
    fn get_type(&self, path: &'m ir::Path<'m>) -> Option<&'m ir::TypeDefinition> {
        let m = self.get_module(&path.subpath(1))?;
        m.types.get(path.last())
    }

    /// look up an interface by path
    fn get_interface(&self, path: &'m ir::Path) -> Option<&'m ir::Interface> {
        let m = self.get_module(&path.subpath(1))?;
        m.interfaces.get(path.last())
    }

    /// look up a function by path
    fn get_function(&self, path: &'m ir::Path) -> Option<&'m (ir::FunctionSignature, ir::FnBody)> {
        let m = self.get_module(&path.subpath(1))?;
        m.functions.get(path.last())
    }

    /// look up the implementation function specific to type `ty` for the interface function `interface_fn`
    fn find_impl(&self, interface_fn: &'m ir::Path<'m>, ty: &ir::Type) -> Option<&'m (ir::FunctionSignature, ir::FnBody)> {
        let if_path = interface_fn.subpath(1);
        let fn_name = interface_fn.last();
        let m = self.get_module(&interface_fn.subpath(2))?;
        let fn_sym = m.implementations.get(&(ty.clone(), if_path))
            .and_then(|m| m.get(fn_name))?;
        m.functions.get(fn_sym)
    }

    fn size_of_user_type(&self, td: &ir::TypeDefinition<'m>, params: &Option<Vec<ir::Type<'m>>>) -> Result<usize> {
        if let Some(_) = params {
            todo!("compute the size of a specialized generic type");
        } else {
            match td {
                ir::TypeDefinition::Sum { variants, .. } => {
                    variants.iter().map(|(_, td)| self.size_of_user_type(td, &None))
                        .fold_ok(0, |a, b| a.max(b))
                },
                ir::TypeDefinition::Product { fields, .. } => {
                    fields.iter().map(|(_,t)| self.size_of_type(t)).fold_ok(0, std::ops::Add::add)
                }
                ir::TypeDefinition::NewType(t) => self.size_of_type(t),
            }
        }
    }

    /// get the size this type would take in bytes
    fn size_of_type(&self, ty: &ir::Type) -> Result<usize> {
        use ir::Type;
        Ok(match ty {
            Type::Unit => 0,
            Type::Bool => 1,
            Type::Int { width, .. } => *width as usize / 8,
            Type::Float { width } => *width as usize / 8,
            Type::Ref(_) | Type::AbstractRef(_) | Type::Array(_) => std::mem::size_of::<*mut usize>(),
            Type::Tuple(t) => t.iter().map(|t| self.size_of_type(t)).fold_ok(0, std::ops::Add::add)?,
            Type::User(def_path, params) => self.get_type(def_path).ok_or_else(|| anyhow!(""))
                .and_then(|t| self.size_of_user_type(t, params))?,
            Type::FnRef(_) => 0, //for now not sure what we'll actually store here
            Type::Var(_) => panic!(),
        })
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

impl Integer {
    fn width(&self) -> u8 {
        match self {
            Integer::U8(_) | Integer::S8(_) => 8,
            Integer::U16(_) | Integer::S16(_) => 16,
            Integer::U32(_) | Integer::S32(_) => 32,
            Integer::U64(_) | Integer::S64(_) => 64,
        }
    }

    fn signed(&self) -> bool {
        match self {
            Integer::U8(_) | Integer::U16(_) | Integer::U32(_) | Integer::U64(_) => false,
            Integer::S8(_) | Integer::S16(_) | Integer::S32(_) | Integer::S64(_) => true,
        }
    }
}


#[derive(Clone, Debug)]
enum Float {
    F32(f32),
    F64(f64)
}

impl Float {
    fn width(&self) -> u8 {
        match self {
            Float::F32(_) => 32,
            Float::F64(_) => 64,
        }
    }
}


/// Values have an implicit lifetime tied to the Heap they were allocated on
#[derive(Clone, Debug)]
enum Value {
    Nil, Bool(bool),
    Int(Integer),
    Float(Float),
    Ref(*mut Value), Array(*mut Value), Fn
}

impl Value {
    fn type_of<'w>(&self, mem: &Memory<'w>) -> ir::Type<'w> {
        match self {
            Value::Nil => ir::Type::Unit,
            Value::Bool(_) => ir::Type::Bool,
            Value::Int(i) => ir::Type::Int { signed: i.signed(), width: i.width() },
            Value::Float(f) => ir::Type::Float { width: f.width() },
            Value::Ref(v) => mem.type_for(*v),
            Value::Array(v) => mem.type_for(*v),
            Value::Fn => ir::Type::FnRef(todo!()),
        }
    }
}

mod mem {
    pub struct Header<'m> {
        pub ty: Box<ir::Type<'m>>,
        pub elements: usize,
        pub prev: *mut Header<'m>
    }
}

struct Memory<'w> {
    world: &'w World<'w>,
    last_alloc: *mut mem::Header<'w>,
    max_size: usize, current_size: usize,

    stack: Vec<Frame>
}

use std::alloc::Layout;

impl<'w> Memory<'w> {
    fn new(world:&'w World<'w>) -> Memory<'w> {
        Memory {
            world, last_alloc: std::ptr::null_mut(),
            max_size: 4 * 1024 * 1024 * 1024, // 4GiB
            current_size: 0,
            stack: Vec::new()
        }
    }

    /// allocate a new value on the heap, and return a reference value
    fn alloc(&mut self, ty: &ir::Type<'w>) -> Result<Value> {
        if let ir::Type::Array(_) = ty {
            bail!("use alloc_array to allocate arrays");
        }

        let mut ran_gc = false;
        loop {
            let layout = Layout::from_size_align(std::mem::size_of::<mem::Header>() + self.world.size_of_type(ty)?, 8)?;
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
                let mem = std::alloc::alloc(layout) as *mut mem::Header;
                (&mut *mem).ty = Box::new(ty.clone());
                (&mut *mem).elements = 1;
                // make sure we can still find this allocation if there aren't any other references to
                // it when we do garbage collection
                (&mut *mem).prev = self.last_alloc;
                self.last_alloc = mem;
                self.current_size += layout.size();
                // should this be aligned?
                return Ok(Value::Ref(mem.offset(std::mem::size_of::<mem::Header>() as isize) as *mut Value));
            }
        }
    }

    fn alloc_array(&mut self, el_ty: &ir::Type<'w>, count: usize) -> Result<Value> {
        let mut ran_gc = false;
        loop {
            let layout = Layout::from_size_align(std::mem::size_of::<mem::Header>() + self.world.size_of_type(el_ty)?*count, 8)?;
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
                let mem = std::alloc::alloc(layout) as *mut mem::Header;
                (&mut *mem).ty = Box::new(el_ty.clone());
                (&mut *mem).elements = count;
                // make sure we can still find this allocation if there aren't any other references to
                // it when we do garbage collection
                (&mut *mem).prev = self.last_alloc;
                self.last_alloc = mem;
                self.current_size += layout.size();
                // should this be aligned?
                return Ok(Value::Array(mem.offset(std::mem::size_of::<mem::Header>() as isize) as *mut Value));
            }
        }
    }

    fn type_for(&self, rf: *mut Value) -> ir::Type<'w> {
        let header = unsafe {&*(rf.offset(-(std::mem::size_of::<mem::Header>() as isize)) as *mut mem::Header)};
        *(header.ty.clone())
    }

    fn element_count(&self, rf: *mut Value) -> usize {
        let header = unsafe {&*(rf.offset(-(std::mem::size_of::<mem::Header>() as isize)) as *mut mem::Header)};
        header.elements
    }

    /// move a value into the heap, returning a reference value
    fn box_value(&mut self, val: Value) -> Result<Value> {
        match self.alloc(&val.type_of(self))? {
            Value::Ref(r) => {
                unsafe { *r = val; }
                Ok(Value::Ref(r))
            }
            _ => unreachable!()
        }
    }

    /// run garbage collection
    fn gc(&mut self) {
        // gc needs access to both the stack and heap to know what is alive
        log::info!("running garbage collection. current size={}, max size={}", self.current_size, self.max_size);
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
        let start_sym = &"start".into();
        let (_, body) = self.world.get_function(&start_sym)
            .expect("a start function is present");
        let _ = self.call_fn(body, vec![]).unwrap();
    }

    /// look up and call a function by interpreting its body to determine the return value
    fn call_fn(&mut self, body: &ir::FnBody, args: Vec<Value>) -> Result<Value> {
        self.mem.stack.push(Frame::new(body.max_registers as usize));
        let cur_frame = self.mem.stack.last().unwrap();
        todo!()
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
