use crate::BlockId;

#[derive(Clone, Debug)]
pub enum BlockValue {
    Undefined,
    String(String),
    Number(f64),
    Boolean(bool),
    BlockId(BlockId),
}

impl BlockValue {
    pub fn to_number(&self) -> f64 {
        match self {
            Self::Number(v) => *v,
            Self::String(v) => str::parse::<f64>(v.trim())
                .or_else(|_| str::parse::<i64>(v.trim()).map(|v| v as f64))
                .unwrap_or(0.),
            Self::Boolean(v) => {
                if *v {
                    1.
                } else {
                    0.
                }
            }
            _ => 0.,
        }
    }

    pub fn to_number_raw(&self) -> f64 {
        match self {
            Self::Number(v) => *v,
            Self::String(v) => str::parse::<f64>(v.trim())
                .or_else(|_| str::parse::<i64>(v.trim()).map(|v| v as f64))
                .unwrap_or(f64::NAN),
            Self::Boolean(v) => {
                if *v {
                    1.
                } else {
                    0.
                }
            }
            _ => f64::NAN,
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Self::Boolean(v) => *v,
            Self::Number(v) => *v >= f64::EPSILON,
            Self::String(v) => {
                !(v.as_str() == "" || v.as_str() == "0" || v.to_lowercase().as_str() == "false")
            }
            _ => false,
        }
    }

    pub fn is_block(&self) -> bool {
        matches!(self, Self::BlockId(_))
    }

    pub fn is_white_space(&self) -> bool {
        match self {
            Self::Undefined => false,
            Self::String(v) => v.trim().is_empty(),
            _ => false,
        }
    }
}

impl std::fmt::Display for BlockValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(v) => f.write_str(v),
            Self::Number(v) => v.fmt(f),
            Self::Boolean(v) => v.fmt(f),
            _ => f.write_str("undefined"),
        }
    }
}

impl std::cmp::PartialEq for BlockValue {
    fn eq(&self, other: &Self) -> bool {
        let mut a = self.to_number_raw();
        let mut b = other.to_number_raw();
        if a == 0. && (self.is_white_space()) {
            a = f64::NAN;
        } else if b == 0. && (other.is_white_space()) {
            b = f64::NAN;
        }
        if a.is_nan() || b.is_nan() {
            if let Self::Boolean(v) = self {
                if let Self::Number(v2) = other {
                    if v2 == &0. || v2 == &1. {
                        return !(v == &false && v2 == &0.);
                    }
                } else if let Self::String(v2) = other {
                    let trimed = v2.trim();
                    if trimed == "false" || trimed == "true" {
                        return (v == &false && trimed == "false")
                            || (v == &true && trimed == "true");
                    }
                }
            }
            if let Self::Number(v) = self {
                if v == &0. || v == &1. {
                    if let Self::Boolean(v2) = other {
                        return !(v == &0. && v2 == &false);
                    } else if let Self::String(v2) = other {
                        let trimed = v2.trim();
                        if trimed == "false" || trimed == "true" {
                            return (v == &0. && trimed == "false")
                                || (v == &1. && trimed == "true");
                        }
                    }
                }
            }
            if let Self::String(v) = self {
                let trimed = v.trim();
                if trimed == "false" || trimed == "true" {
                    if let Self::Boolean(v2) = other {
                        return (v == "false" && v2 == &false) || (v == "true" && v2 == &true);
                    } else if let Self::Number(v2) = other {
                        return (v == "false" && v2 == &0.) || (v == "true" && v2 == &1.);
                    }
                }
            }
            let a = self.to_string();
            let b = other.to_string();
            a.eq_ignore_ascii_case(b.as_str())
        } else {
            (a - b).abs() < f64::EPSILON
        }
    }
}

impl std::cmp::PartialOrd for BlockValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut a = self.to_number_raw();
        let mut b = other.to_number_raw();
        if a == 0. && self.is_white_space() {
            a = f64::NAN;
        } else if b == 0. && other.is_white_space() {
            b = f64::NAN;
        }
        if a.is_nan() || b.is_nan() {
            if let Self::Boolean(v) = self {
                if let Self::Number(v2) = other {
                    if !(v == &false && v2 == &0.) {
                        return Some(std::cmp::Ordering::Equal);
                    }
                }
            }
            if let Self::Number(v) = self {
                if let Self::Boolean(v2) = other {
                    if !(v == &0. && v2 == &false) {
                        return Some(std::cmp::Ordering::Equal);
                    }
                }
            }
            let mut a = self.to_string();
            let mut b = other.to_string();
            a.make_ascii_lowercase();
            b.make_ascii_lowercase();
            a.partial_cmp(&b)
        } else {
            a.partial_cmp(&b)
        }
    }
}

impl From<&str> for BlockValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_owned())
    }
}

impl From<String> for BlockValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<bool> for BlockValue {
    fn from(v: bool) -> Self {
        Self::Boolean(v as _)
    }
}

impl From<i8> for BlockValue {
    fn from(v: i8) -> Self {
        Self::Number(v as _)
    }
}

impl From<i16> for BlockValue {
    fn from(v: i16) -> Self {
        Self::Number(v as _)
    }
}

impl From<i32> for BlockValue {
    fn from(v: i32) -> Self {
        Self::Number(v as _)
    }
}

impl From<i64> for BlockValue {
    fn from(v: i64) -> Self {
        Self::Number(v as _)
    }
}

impl From<i128> for BlockValue {
    fn from(v: i128) -> Self {
        Self::Number(v as _)
    }
}

impl From<u8> for BlockValue {
    fn from(v: u8) -> Self {
        Self::Number(v as _)
    }
}

impl From<u16> for BlockValue {
    fn from(v: u16) -> Self {
        Self::Number(v as _)
    }
}

impl From<u32> for BlockValue {
    fn from(v: u32) -> Self {
        Self::Number(v as _)
    }
}

impl From<u64> for BlockValue {
    fn from(v: u64) -> Self {
        Self::Number(v as _)
    }
}

impl From<u128> for BlockValue {
    fn from(v: u128) -> Self {
        Self::Number(v as _)
    }
}

impl From<isize> for BlockValue {
    fn from(v: isize) -> Self {
        Self::Number(v as _)
    }
}

impl From<usize> for BlockValue {
    fn from(v: usize) -> Self {
        Self::Number(v as _)
    }
}

impl From<f32> for BlockValue {
    fn from(v: f32) -> Self {
        if v.is_nan() {
            Self::Number(0.)
        } else {
            Self::Number(v as _)
        }
    }
}

impl From<f64> for BlockValue {
    fn from(v: f64) -> Self {
        if v.is_nan() {
            Self::Number(0.)
        } else {
            Self::Number(v as _)
        }
    }
}

impl Default for BlockValue {
    fn default() -> Self {
        Self::Undefined
    }
}
