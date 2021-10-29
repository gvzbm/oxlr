use std::{borrow::Cow, collections::HashMap, fmt::Display};
use serde::{Serialize, Deserialize};
pub use semver::{Version, VersionReq};

mod code;
pub use code::FnBody;

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct Symbol<'a>(pub Cow<'a, str>);

/// A full path of module, submodules and optionally final name of a type/interface/function, based
/// on context
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct Path<'a>(pub Vec<Symbol<'a>>);


#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub enum Type<'a> {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TypeDefinition<'a> {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interface<'a> {
    pub name: Symbol<'a>,
    pub functions: HashMap<Symbol<'a>, FunctionSignature<'a>>
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct FunctionSignature<'a> {
    pub args: Vec<(Type<'a>, Symbol<'a>)>,
    pub return_type: Type<'a>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Module<'a> {
    pub path: Path<'a>,
    pub version: Version,
    pub types: HashMap<Symbol<'a>, TypeDefinition<'a>>,
    pub interfaces: HashMap<Symbol<'a>, Interface<'a>>,
    /// (type, interface path) -> specific function names for implementation functions provided in this module indexed by the interface function they implement
    pub implementations: HashMap<(Type, Path), HashMap<Symbol, Symbol>>,
    pub functions: HashMap<Symbol, (FunctionSignature, FnBody)>,
    pub imports: Vec<(Path, VersionReq)>,
}


impl<'a> Path<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item=&Symbol> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Symbol<'a>> {
        self.0.iter_mut()
    }

    pub fn subpath(&self, len: usize) -> Path {
        Path(self.0[0..self.0.len()-len].to_vec())
    }

    pub fn last(&self) -> &Symbol {
        self.0.last().expect("paths must have at least one element")
    }
}

impl<'a, T: 'a + AsRef<str>> From<T> for Path<'a> {
    fn from(s: T) -> Self {
        Path(s.as_ref().split("::").map(|s| Symbol(Cow::Owned(s.to_owned()))).collect())
    }
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for s in self.iter().take(self.len()-1) {
            f.write_str(&s.0)?;
            f.write_str("::")?;
        }
        f.write_str(&self.0.last().unwrap().0)
    }
}

impl<'a> std::ops::Index<usize> for Path<'a> {
    type Output = Symbol<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a> std::ops::IndexMut<usize> for Path<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

