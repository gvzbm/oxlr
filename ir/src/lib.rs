use std::{borrow::Cow, collections::HashMap, fmt::Display};
use serde::{Serialize, Deserialize};
pub use semver::{Version, VersionReq};

pub mod code;
pub use code::FnBody;

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct Symbol(pub String);

/// A full path of module, submodules and optionally final name of a type/interface/function, based
/// on context
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct Path(pub Vec<Symbol>);

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub enum Type {
    Unit,
    Bool,
    Int { signed: bool, width: u8 },
    Float { width: u8 },
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TypeDefinition {
    NewType(Type),
    Sum {
        /// (name of parameter, list of interfaces it must implement)
        parameters: Vec<(Symbol, Vec<Path>)>,
        variants: Vec<(Symbol, TypeDefinition)>
    },
    Product {
        /// (name of parameter, list of interfaces it must implement)
        parameters: Vec<(Symbol, Vec<Path>)>,
        fields: Vec<(Symbol, Type)>
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interface {
    pub name: Symbol,
    pub functions: HashMap<Symbol, FunctionSignature>
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct FunctionSignature {
    pub args: Vec<(Type, Symbol)>,
    pub return_type: Type
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Module {
    pub path: Path,
    pub version: Version,
    pub types: HashMap<Symbol, TypeDefinition>,
    pub interfaces: HashMap<Symbol, Interface>,
    /// (type, interface path) -> specific function names for implementation functions provided in this module indexed by the interface function they implement
    pub implementations: HashMap<(Type, Path), HashMap<Symbol, Symbol>>,
    pub functions: HashMap<Symbol, (FunctionSignature, FnBody)>,
    pub imports: Vec<(Path, VersionReq)>,
}


impl Path {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item=&Symbol> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Symbol> {
        self.0.iter_mut()
    }

    pub fn subpath(&self, len: usize) -> Path {
        Path(self.0[0..self.0.len()-len].to_vec())
    }

    pub fn last(&self) -> &Symbol {
        self.0.last().expect("paths must have at least one element")
    }
}

impl<'a, T: 'a + AsRef<str>> From<T> for Path {
    fn from(s: T) -> Self {
        Path(s.as_ref().split("::").map(|s| Symbol(s.to_string())).collect())
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for s in self.iter().take(self.len()-1) {
            f.write_str(&s.0)?;
            f.write_str("::")?;
        }
        f.write_str(&self.0.last().unwrap().0)
    }
}

impl std::ops::Index<usize> for Path {
    type Output = Symbol;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Path {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
