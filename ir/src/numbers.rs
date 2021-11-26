use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Integer {
    pub width: u8, pub signed: bool,
    pub data: u64
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Float {
    F32(f32),
    F64(f64)
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

macro_rules! integer_impl_from {
    ( $sfn:ident, $(($w:expr, $it:ty)),* ) => {
        $(
            impl From<$it> for Integer {
                fn from(x: $it) -> Integer {
                    Integer::$sfn($w, x as u64)
                }
            }
        )*
    }
}

integer_impl_from!(unsigned, (8,u8), (16,u16), (32,u32), (64,u64));
integer_impl_from!(signed, (8,i8), (16,i16), (32,i32), (64,i64));

impl Float {
    pub fn width(&self) -> u8 {
        match self {
            Float::F32(_) => 32,
            Float::F64(_) => 64,
        }
    }
}


