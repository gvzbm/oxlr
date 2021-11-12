
#[derive(Clone, Debug)]
pub enum Integer {
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
    pub fn width(&self) -> u8 {
        match self {
            Integer::U8(_) | Integer::S8(_) => 8,
            Integer::U16(_) | Integer::S16(_) => 16,
            Integer::U32(_) | Integer::S32(_) => 32,
            Integer::U64(_) | Integer::S64(_) => 64,
        }
    }

    pub fn signed(&self) -> bool {
        match self {
            Integer::U8(_) | Integer::U16(_) | Integer::U32(_) | Integer::U64(_) => false,
            Integer::S8(_) | Integer::S16(_) | Integer::S32(_) | Integer::S64(_) => true,
        }
    }
}


#[derive(Clone, Debug)]
pub enum Float {
    F32(f32),
    F64(f64)
}

impl Float {
    pub fn width(&self) -> u8 {
        match self {
            Float::F32(_) => 32,
            Float::F64(_) => 64,
        }
    }
}

/// Values have an implicit lifetime tied to the Heap they were allocated on
#[derive(Clone, Debug)]
pub enum Value {
    Nil, Bool(bool),
    Int(Integer),
    Float(Float),
    Ref(*mut Value), Array(*mut Value), Fn
}

impl Value {
    pub fn type_of<'w>(&self, mem: &crate::memory::Memory<'w>) -> ir::Type<'w> {
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


