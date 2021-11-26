pub use ir::{Integer, Float};

/// Values have an implicit lifetime tied to the Heap they were allocated on
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil, Bool(bool),
    Int(Integer),
    Float(Float),
    Ref(crate::memory::HeapRef), Array(crate::memory::HeapRef), Fn
}

impl Value {
    pub fn type_of(&self, mem: &crate::memory::Memory) -> ir::Type {
        match self {
            Value::Nil => ir::Type::Unit,
            Value::Bool(_) => ir::Type::Bool,
            Value::Int(i) => ir::Type::Int { signed: i.signed, width: i.width },
            Value::Float(f) => ir::Type::Float { width: f.width() },
            Value::Ref(v) => v.type_of().clone(),
            Value::Array(v) => v.type_of().clone(),
            Value::Fn => ir::Type::FnRef(todo!()),
        }
    }
}
