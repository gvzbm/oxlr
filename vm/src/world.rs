use std::{collections::HashMap};
use itertools::Itertools;
use anyhow::*;

pub struct World {
    global_module_path: std::path::PathBuf,
    local_module_path: std::path::PathBuf,
    modules: HashMap<ir::Path, ir::Module>,
    instantiated_types: HashMap<(ir::Path, Vec<ir::Type>), ir::TypeDefinition>
}

fn test_module_file_candidate(dir_entry: &std::fs::DirEntry, path: &ir::Path, version_req: &ir::VersionReq) -> Option<std::path::PathBuf> {
    let mut filename = dir_entry.file_name().into_string().ok()?;
    if !filename.ends_with(".om") {
        return None;
    } else {
        filename.truncate(filename.len() - 3)
    }
    log::trace!("testing {} as candidate for module", dir_entry.path().display());
    let (fpath, fver) = filename.split_once('#')?;
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

impl World {
    pub fn new() -> Result<World> {
        Ok(World {
            global_module_path: std::env::var("OXLR_MODULE_PATH").map(std::path::PathBuf::from)?,
            local_module_path: std::env::current_dir()?,
            modules: HashMap::new(),
            instantiated_types: HashMap::new()
        })
    }

    /// get a module, loading it from the filesystem if necessary by searching the module search paths
    pub fn load_module<'s>(&mut self, path: &'s ir::Path, version: &ir::VersionReq) -> Result<()> {
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

    pub fn get_module(&self, path: &ir::Path) -> Option<&ir::Module> {
        self.modules.get(path)
    }

    /// look up a type definition by path
    pub fn get_type(&self, path: &ir::Path) -> Option<&ir::TypeDefinition> {
        let m = self.get_module(&path.subpath(1))?;
        m.types.get(path.last())
    }

    /// look up an interface by path
    pub fn get_interface(&self, path: &ir::Path) -> Option<&ir::Interface> {
        let m = self.get_module(&path.subpath(1))?;
        m.interfaces.get(path.last())
    }

    /// look up a function by path
    pub fn get_function(&self, path: &ir::Path) -> Option<&(ir::FunctionSignature, ir::FnBody)> {
        let m = self.get_module(&path.subpath(1))?;
        m.functions.get(path.last())
    }

    /// look up the implementation function specific to type `ty` for the interface function `interface_fn`
    pub fn find_impl(&self, interface_fn: &ir::Path, ty: &ir::Type) -> Option<&(ir::FunctionSignature, ir::FnBody)> {
        let if_path = interface_fn.subpath(1);
        let fn_name = interface_fn.last();
        let m = self.get_module(&interface_fn.subpath(2))?;
        let fn_sym = m.implementations.get(&(ty.clone(), if_path))
            .and_then(|m| m.get(fn_name))?;
        m.functions.get(fn_sym)
    }

    pub fn size_of_user_type(&self, td: &ir::TypeDefinition, params: &Option<Vec<ir::Type>>) -> Result<usize> {
        if let Some(_) = params {
            todo!("compute the size of a specialized generic type");
        } else {
            match td {
                ir::TypeDefinition::Sum { variants, .. } => {
                    variants.iter().map(|(_, td)| self.size_of_user_type(td, &None))
                        .fold_ok(0, |a, b| a.max(b))
                },
                ir::TypeDefinition::Product { fields, .. } => {
                    let mut size = 0;
                    for (_, ty) in fields.iter() {
                        let ralign = self.required_alignment(ty)?;
                        while size % ralign != 0 { size += 1; }
                        size += self.size_of_type(ty)?;
                    }
                    Ok(size)
                }
                ir::TypeDefinition::NewType(t) => self.size_of_type(t),
            }
        }
    }

    /// get the size this type would take in bytes
    pub fn size_of_type(&self, ty: &ir::Type) -> Result<usize> {
        use ir::Type;
        Ok(match ty {
            Type::Unit => 0,
            Type::Bool => 1,
            Type::Int { width, .. } => *width as usize / 8,
            Type::Float { width } => *width as usize / 8,
            Type::Ref(_) | Type::AbstractRef(_) | Type::Array(_) => std::mem::size_of::<crate::memory::Ref>(),
            Type::Tuple(fields) => {
                let mut size = 0;
                for ty in fields.iter() {
                    let ralign = self.required_alignment(ty)?;
                    while size % ralign != 0 { size += 1; }
                    size += self.size_of_type(ty)?;
                }
                size
            },
            Type::User(def_path, params) => self.get_type(def_path)
                .ok_or_else(|| anyhow!("unknown type ty={:?}", ty))
                .and_then(|t| self.size_of_user_type(t, params))?,
            Type::FnRef(_) => 0, //for now not sure what we'll actually store here
            Type::Var(_) => panic!(),
        })
    }

    pub fn required_alignment(&self, ty: &ir::Type) -> Result<usize> {
        use ir::Type;
        Ok(match ty {
            Type::Unit => 1,
            Type::Bool => 1,
            Type::Int { width, .. } => *width as usize / 8,
            Type::Float { width } => *width as usize / 8,
            // TODO: for now, everything gets aligned to the pointer alignment... this is a good
            // guess, but is it the correct one? I'm not sure
            Type::Ref(_) | Type::AbstractRef(_) | Type::Array(_)
                | Type::User(_,_) | Type::Tuple(_) | Type::FnRef(_) => std::mem::align_of::<*mut u8>(),
            Type::Var(_) => panic!(),
        })
    }
}
