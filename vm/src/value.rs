
//TODO: this is a mess, but we need to have a way to combine all the different widths
//  due to the way Rust works, this probably winds up being more than 8 bytes no matter what,
//  so maybe it would make more sense to just keep track of the width/sign and value rather than using an enum
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Float {
    F32(f32),
    F64(f64)
}

/// Values have an implicit lifetime tied to the Heap they were allocated on
#[derive(Clone, Debug, PartialEq)]
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

    pub fn bitwise_negate(&self) -> Integer {
        match self {
            Integer::U8(x) => Integer::U8(!x),
            Integer::U16(x) => Integer::U16(!x),
            Integer::U32(x) => Integer::U32(!x),
            Integer::U64(x) => Integer::U64(!x),
            Integer::S8(x) => Integer::S8(!x),
            Integer::S16(x) => Integer::S16(!x),
            Integer::S32(x) => Integer::S32(!x),
            Integer::S64(x) => Integer::S64(!x),
        }
    }

    pub fn negate(&self) -> Integer {
        match self {
            Integer::S8(x) => Integer::S8(-x),
            Integer::S16(x) => Integer::S16(-x),
            Integer::S32(x) => Integer::S32(-x),
            Integer::S64(x) => Integer::S64(-x),
            _ => panic!()
        }
    }
}

impl std::ops::Add for Integer {
    type Output = Integer;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(x), Integer::U8(y)) => Integer::U8(x+y),
            (Integer::U16(x), Integer::U16(y)) => Integer::U16(x+y),
            (Integer::U32(x), Integer::U32(y)) => Integer::U32(x+y),
            (Integer::U64(x), Integer::U64(y)) => Integer::U64(x+y),
            (Integer::S8(x), Integer::S8(y)) => Integer::S8(x+y),
            (Integer::S16(x), Integer::S16(y)) => Integer::S16(x+y),
            (Integer::S32(x), Integer::S32(y)) => Integer::S32(x+y),
            (Integer::S64(x), Integer::S64(y)) => Integer::S64(x+y),
            // TODO: this really shouldn't panic. Ideally either we return a result or
            // something. Probably there could be some implicit widing done as well
            _ => panic!("mismatched integers")
        }
    }
}

impl std::ops::Sub for Integer {
    type Output = Integer;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(x), Integer::U8(y)) => Integer::U8(x-y),
            (Integer::U16(x), Integer::U16(y)) => Integer::U16(x-y),
            (Integer::U32(x), Integer::U32(y)) => Integer::U32(x-y),
            (Integer::U64(x), Integer::U64(y)) => Integer::U64(x-y),
            (Integer::S8(x), Integer::S8(y)) => Integer::S8(x-y),
            (Integer::S16(x), Integer::S16(y)) => Integer::S16(x-y),
            (Integer::S32(x), Integer::S32(y)) => Integer::S32(x-y),
            (Integer::S64(x), Integer::S64(y)) => Integer::S64(x-y),
            // TODO: like addition. also these can over/underflow and right now they panic
            _ => panic!("mismatched integers")
        }
    }
}

impl std::ops::Mul for Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(x), Integer::U8(y)) => Integer::U8(x*y),
            (Integer::U16(x), Integer::U16(y)) => Integer::U16(x*y),
            (Integer::U32(x), Integer::U32(y)) => Integer::U32(x*y),
            (Integer::U64(x), Integer::U64(y)) => Integer::U64(x*y),
            (Integer::S8(x), Integer::S8(y)) => Integer::S8(x*y),
            (Integer::S16(x), Integer::S16(y)) => Integer::S16(x*y),
            (Integer::S32(x), Integer::S32(y)) => Integer::S32(x*y),
            (Integer::S64(x), Integer::S64(y)) => Integer::S64(x*y),
            // TODO: like addition. also these can over/underflow and right now they panic
            _ => panic!("mismatched integers")
        }
    }
}

impl std::ops::Div for Integer {
    type Output = Integer;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(x), Integer::U8(y)) => Integer::U8(x/y),
            (Integer::U16(x), Integer::U16(y)) => Integer::U16(x/y),
            (Integer::U32(x), Integer::U32(y)) => Integer::U32(x/y),
            (Integer::U64(x), Integer::U64(y)) => Integer::U64(x/y),
            (Integer::S8(x), Integer::S8(y)) => Integer::S8(x/y),
            (Integer::S16(x), Integer::S16(y)) => Integer::S16(x/y),
            (Integer::S32(x), Integer::S32(y)) => Integer::S32(x/y),
            (Integer::S64(x), Integer::S64(y)) => Integer::S64(x/y),
            // TODO: like addition. also these can over/underflow and right now they panic
            _ => panic!("mismatched integers")
        }
    }
}

impl Float {
    pub fn width(&self) -> u8 {
        match self {
            Float::F32(_) => 32,
            Float::F64(_) => 64,
        }
    }
}


