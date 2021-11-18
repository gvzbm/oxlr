
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Integer {
    pub width: u8, pub signed: bool,
    pub data: u64
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

impl Integer {
    pub fn new(width: u8, signed: bool, data: u64) -> Integer {
        Integer { width, signed, data }
    }

    pub fn unsigned(width: u8, data: u64) -> Integer {
        Integer { width, signed: false, data }
    }
    pub fn signed(width: u8, data: u64) -> Integer {
        Integer { width, signed: true, data }
    }

    pub fn bitwise_negate(&self) -> Integer {
        Integer {
            data: !self.data,
            ..*self
        }
    }

    pub fn negate(&self) -> Integer {
        assert!(self.signed);
        Integer {
            data: !self.data + 1,
            ..*self
        }
    }
}

impl std::ops::Add for Integer {
    type Output = Integer;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(self.width >= rhs.width);
        assert!(self.signed == rhs.signed);
        Integer {
            data: self.data + rhs.data,
            ..self
        }
    }
}

impl std::ops::Sub for Integer {
    type Output = Integer;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.width >= rhs.width);
        assert!(self.signed == rhs.signed);
        Integer {
            data: self.data - rhs.data,
            ..self
        }
    }
}

impl std::ops::Mul for Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        assert!(self.width >= rhs.width);
        assert!(self.signed == rhs.signed);
        Integer {
            data: self.data * rhs.data,
            ..self
        }
    }
}

impl std::ops::Div for Integer {
    type Output = Integer;

    fn div(self, rhs: Self) -> Self::Output {
        assert!(self.width >= rhs.width);
        assert!(self.signed == rhs.signed);
        Integer {
            data: self.data / rhs.data,
            ..self
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


