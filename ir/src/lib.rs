use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use semver::{Version, VersionReq};

mod code;
pub use code::FnBody;

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct Symbol(String);

/// A full path of module, submodules and optionally final name of a type/interface/function, based
/// on context
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct Path(Vec<Symbol>);

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub enum Type {
    Unit,
    Bool,
    Int { signed: bool, width: u8 },
    Float { width: u8 },
    String,
    Array(Box<Type>),
    Tuple(Vec<Type>),
    /// (the type definition, any type parameters)
    User(Path, Option<Vec<Type>>),
    Ref(Box<Type>),
    /// like Rust's &dyn A + B + C
    AbstractRef(Vec<Path>),
    FnRef(Box<FunctionSignature>),
    /// A reference to a type parameter inside a definition
    Var(Symbol)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TypeDefinition {
    Sum {
        name: Symbol,
        /// (name of parameter, list of interfaces it must implement)
        parameters: Vec<(Symbol, Vec<Path>)>,
        variants: Vec<(Symbol, TypeDefinition)>
    },
    Product {
        name: Symbol,
        /// (name of parameter, list of interfaces it must implement)
        parameters: Vec<(Symbol, Vec<Path>)>,
        fields: Vec<(Symbol, Type)>
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Interface {
    pub name: Symbol,
    pub functions: HashMap<Symbol, FunctionSignature>
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct FunctionSignature {
    pub args: Vec<(Type, Symbol)>,
    pub return_type: Type
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Module {
    pub name: String,
    pub version: Version,
    pub submodules: Vec<Module>,
    pub types: HashMap<Symbol, TypeDefinition>,
    pub interfaces: HashMap<Symbol, Interface>,
    /// (type, interface path) -> specific function names for implementation functions provided in this module indexed by the interface function they implement
    pub implementations: HashMap<(Type, Path), HashMap<Symbol, Symbol>>,
    pub functions: HashMap<Symbol, (FunctionSignature, FnBody)>,
    pub imports: Vec<(Path, VersionReq)>,
}

