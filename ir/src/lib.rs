use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use semver::{Version, VersionReq};

mod code;
pub use code::FnBody;

/// A full path of module, submodules and optionally final name of a type/interface/function, based
/// on context
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct Path(Vec<String>);

#[derive(Serialize, Deserialize, Debug)]
pub enum Type {
    Unit,
    Bool,
    Int { width: u8 },
    Float { width: u8 },
    String,
    Tuple(Vec<Type>),
    User(Path),
    Ref(Box<Type>)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TypeDefinition {
    Sum {
        name: String,
        variants: Vec<(String, TypeDefinition)>
    },
    Product {
        name: String,
        fields: Vec<(String, Type)>
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Interface {
    pub name: String,
    pub functions: Vec<FunctionSignature>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub args: Vec<(Type, String)>,
    pub return_type: Type
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Module {
    pub name: String,
    pub version: Version,
    pub submodules: Vec<Module>,
    pub types: Vec<TypeDefinition>,
    pub interfaces: Vec<Interface>,
    // This is a map from (type path, interface path) to a list of function bodies implementing
    // each interface function, indexed by name
    pub implementations: HashMap<(Path, Path), HashMap<String, FnBody>>,
    pub functions: Vec<(FunctionSignature, FnBody)>,
    pub imports: Vec<(Path, VersionReq)>
}
