use std::{borrow::Cow, collections::HashMap};
use serde::{Serialize, Deserialize};
use semver::{Version, VersionReq};

pub mod code;
pub use code::FnBody;

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct Symbol<'a>(Cow<'a, str>);

/// A full path of module, submodules and optionally final name of a type/interface/function, based
/// on context
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct Path<'a>(Vec<Symbol<'a>>);

impl<'a, T: 'a + AsRef<str>> From<T> for Path<'a> {
    fn from(s: T) -> Self {
        Path(s.as_ref().split("::").map(|s| Symbol(Cow::Owned(s.to_owned()))).collect())
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub enum Type<'a> {
    Unit,
    Bool,
    Int { signed: bool, width: u8 },
    Float { width: u8 },
    String,
    Array(Box<Type<'a>>),
    Tuple(Vec<Type<'a>>),
    /// (the type definition, any type parameters)
    User(Path<'a>, Option<Vec<Type<'a>>>),
    Ref(Box<Type<'a>>),
    /// like Rust's &dyn A + B + C
    AbstractRef(Vec<Path<'a>>),
    FnRef(Box<FunctionSignature<'a>>),
    /// A reference to a type parameter inside a definition
    Var(Symbol<'a>)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TypeDefinition<'a> {
    Sum {
        name: Symbol<'a>,
        /// (name of parameter, list of interfaces it must implement)
        parameters: Vec<(Symbol<'a>, Vec<Path<'a>>)>,
        variants: Vec<(Symbol<'a>, TypeDefinition<'a>)>
    },
    Product {
        name: Symbol<'a>,
        /// (name of parameter, list of interfaces it must implement)
        parameters: Vec<(Symbol<'a>, Vec<Path<'a>>)>,
        fields: Vec<(Symbol<'a>, Type<'a>)>
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Interface<'a> {
    pub name: Symbol<'a>,
    pub functions: HashMap<Symbol<'a>, FunctionSignature<'a>>
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct FunctionSignature<'a> {
    pub args: Vec<(Type<'a>, Symbol<'a>)>,
    pub return_type: Type<'a>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Module<'a> {
    pub name: Cow<'a, str>,
    pub version: Version,
    pub submodules: Vec<Module<'a>>,
    pub types: HashMap<Symbol<'a>, TypeDefinition<'a>>,
    pub interfaces: HashMap<Symbol<'a>, Interface<'a>>,
    /// (type, interface path) -> specific function names for implementation functions provided in this module indexed by the interface function they implement
    pub implementations: HashMap<(Type<'a>, Path<'a>), HashMap<Symbol<'a>, Symbol<'a>>>,
    pub functions: HashMap<Symbol<'a>, (FunctionSignature<'a>, FnBody<'a>)>,
    pub imports: Vec<(Path<'a>, VersionReq)>,
}

