//! The intermediate representation of an OXLR module is represented by this crate's data
//! structures. Each module is represented by a [`Module`] structure, which can be (de)serialized
//! to/from a file, for usage in other programs. The actual virtual machine code is defined in the
//! [`code`] module, and is stored in single static assignment form.
use std::{borrow::Cow, collections::HashMap, fmt::Display};
use serde::{Serialize, Deserialize};
pub use semver::{Version, VersionReq};

pub mod code;
pub use code::FnBody;

pub mod numbers;
pub use numbers::*;

/// A `Symbol` represents a single name in a module
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct Symbol(pub String);

/// A full path of module, submodules and optionally final name of a type/interface/function, based
/// on context in which it appears
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
//#[serde(from ="String")]
pub struct Path(pub Vec<Symbol>);

/// The type of a value in the system
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub enum Type {
    /// The unit type, which only has a single value (denoted `()` in Rust)
    Unit,
    /// The boolean type of two values `true` and `false`
    Bool,
    /// The integer numerical types of a particular bit width, with the option of being signed
    Int {
        signed: bool,
        /// The width of this type of integer. Support is guaranteed for 8, 16, 32 and 64 bit
        /// integers, but anything else is unlikely to be supported.
        width: u8
    },
    /// A floating point number. Bit widths quantized to 32 and 64 bit floats.
    /// For now the format is unspecified
    Float { width: u8 },
    /// An array of elements of the inner type, with a fixed size determined at runtime
    Array(Box<Type>),
    /// A tuple of elements, in the order their types are specified
    Tuple(Vec<Type>),
    /// A user-defined type (see [`TypeDefinition`])
    User(
        /// Path to the user type definition
        Path,
        /// Optional type parameters, by position
        Option<Vec<Type>>
    ),
    /// A reference to a value stored on the heap
    Ref(Box<Type>),
    /// A reference to a value stored on the heap with the specific type information erased
    /// Instead, each path references an interface that the value stored behind this ref implements
    /// like Rust's &dyn A + B + C
    AbstractRef(Vec<Path>),
    /// A function pointer to a function with the specified signature
    FnRef(Box<FunctionSignature>),
    /// A reference to a type parameter inside a user type definition
    Var(Symbol)
}

/// A user defined type definition
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TypeDefinition {
    /// A single type that gets a new user specified name
    /// Provided to support variants that contain only a single value
    NewType(Type),
    /// A sum type that can contain one of a number of named variants, each with a name and internal type definition
    Sum {
        /// The type parameters required by this type. Vector of (name of parameter, list of interfaces it must implement)
        parameters: Vec<(Symbol, Vec<Path>)>,
        /// Ordered mapping between variant names and their internal type definition
        variants: Vec<(Symbol, TypeDefinition)>
    },
    /// A product type (a struct) that contains a number of named, typed fields
    Product {
        /// The type parameters required by this type. Vector of (name of parameter, list of interfaces it must implement)
        parameters: Vec<(Symbol, Vec<Path>)>,
        fields: Vec<(Symbol, Type)>
    }
}

/// A user defined interface definition
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interface {
    pub name: Symbol,
    /// A mapping from function names to the signature of the function, representing the functions
    /// that implementers of the interface must implement
    pub functions: HashMap<Symbol, FunctionSignature>
}

/// The type signature of a function
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct FunctionSignature {
    pub args: Vec<(Type, Symbol)>,
    pub return_type: Type
}

/// The module represents a contained block of function and type definitions of a specific version
/// This is the root of the IR data structure. Modules can be nested, but this is represented only
/// in their path field as being under the same subpath
#[derive(Serialize, Deserialize, Debug)]
pub struct Module {
    pub path: Path,
    pub version: Version,
    /// The types defined in this module
    pub types: HashMap<Symbol, TypeDefinition>,
    /// The interfaces defined in this module
    pub interfaces: HashMap<Symbol, Interface>,
    /// The interface implementations in this module, indexed by type and interface path.
    /// Implementations can be provided for types and interfaces outside this module.
    /// Mapping of (type, interface path) to specific function names for implementation functions provided in this module indexed by the interface function they implement
    pub implementations: HashMap<(Type, Path), HashMap<Symbol, Symbol>>,
    /// The functions defined in this module outside of any interface implementation
    pub functions: HashMap<Symbol, (FunctionSignature, FnBody)>,
    /// A list of module paths that this module imports, associated with the version requirements for that module that must be met
    pub imports: Vec<(Path, VersionReq)>,
}

impl Path {
    /// The length of this path, counting the number of individual symbols it contains
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Iterator over the symbols in this path
    pub fn iter(&self) -> impl Iterator<Item=&Symbol> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Symbol> {
        self.0.iter_mut()
    }

    /// Returns the path that is a subpath of this one from the beginning, including `len` path symbols
    pub fn subpath(&self, len: usize) -> Path {
        Path(self.0[0..self.0.len()-len].to_vec())
    }

    /// The last symbol in the path. Paths must have at least one element, if an invalid path is
    /// encountered this function will panic
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
