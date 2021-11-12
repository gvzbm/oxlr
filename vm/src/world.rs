use std::{collections::HashMap};
use itertools::Itertools;
use anyhow::*;

pub struct World<'m> {
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
    pub fn new() -> Result<World<'m>> {
        Ok(World {
            global_module_path: std::env::var("OXLR_MODULE_PATH").map(std::path::PathBuf::from)?,
            local_module_path: std::env::current_dir()?,
            modules: HashMap::new(),
            instantiated_types: HashMap::new()
        })
    }

    /// get a module, loading it from the filesystem if necessary by searching the module search paths
    pub fn load_module<'s>(&mut self, path: &'s ir::Path<'m>, version: &ir::VersionReq) -> Result<()> {
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

    pub fn get_module(&self, path: &ir::Path<'m>) -> Option<&ir::Module> {
        self.modules.get(path)
    }

    /// look up a type definition by path
    pub fn get_type(&self, path: &'m ir::Path<'m>) -> Option<&'m ir::TypeDefinition> {
        let m = self.get_module(&path.subpath(1))?;
        m.types.get(path.last())
    }

    /// look up an interface by path
    pub fn get_interface(&self, path: &'m ir::Path) -> Option<&'m ir::Interface> {
        let m = self.get_module(&path.subpath(1))?;
        m.interfaces.get(path.last())
    }

    /// look up a function by path
    pub fn get_function(&self, path: &'m ir::Path) -> Option<&'m (ir::FunctionSignature, ir::FnBody)> {
        let m = self.get_module(&path.subpath(1))?;
        m.functions.get(path.last())
    }

    /// look up the implementation function specific to type `ty` for the interface function `interface_fn`
    pub fn find_impl(&self, interface_fn: &'m ir::Path<'m>, ty: &ir::Type) -> Option<&'m (ir::FunctionSignature, ir::FnBody)> {
        let if_path = interface_fn.subpath(1);
        let fn_name = interface_fn.last();
        let m = self.get_module(&interface_fn.subpath(2))?;
        let fn_sym = m.implementations.get(&(ty.clone(), if_path))
            .and_then(|m| m.get(fn_name))?;
        m.functions.get(fn_sym)
    }

    pub fn size_of_user_type(&self, td: &ir::TypeDefinition<'m>, params: &Option<Vec<ir::Type<'m>>>) -> Result<usize> {
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
    pub fn size_of_type(&self, ty: &ir::Type) -> Result<usize> {
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
