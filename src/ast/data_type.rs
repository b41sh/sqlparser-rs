// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::ast::Ident;
use crate::ast::ObjectName;

/// SQL data types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    /// Fixed-length character type e.g. CHAR(10)
    Char(Option<u64>),
    /// Variable-length character type e.g. VARCHAR(10)
    Varchar(Option<u64>),
    /// Uuid type
    Uuid,
    /// Large character object e.g. CLOB(1000)
    Clob(u64),
    /// Fixed-length binary type e.g. BINARY(10)
    Binary(u64),
    /// Variable-length binary type e.g. VARBINARY(10)
    Varbinary(u64),
    /// Large binary object e.g. BLOB(1000)
    Blob(u64),
    /// Decimal type with optional precision and scale e.g. DECIMAL(10,2)
    Decimal(Option<u64>, Option<u64>),
    /// Floating point with optional precision e.g. FLOAT(8)
    Float(Option<u64>),
    /// Tiny integer with optional display width e.g. TINYINT or TINYINT(3)
    TinyInt(Option<u64>),
    /// Unsigned tiny integer with optional display width e.g. TINYINT UNSIGNED or TINYINT(3) UNSIGNED
    UnsignedTinyInt(Option<u64>),
    /// Small integer with optional display width e.g. SMALLINT or SMALLINT(5)
    SmallInt(Option<u64>),
    /// Unsigned small integer with optional display width e.g. SMALLINT UNSIGNED or SMALLINT(5) UNSIGNED
    UnsignedSmallInt(Option<u64>),
    /// Integer with optional display width e.g. INT or INT(11)
    Int(Option<u64>),
    /// Unsigned integer with optional display width e.g. INT UNSIGNED or INT(11) UNSIGNED
    UnsignedInt(Option<u64>),
    /// Big integer with optional display width e.g. BIGINT or BIGINT(20)
    BigInt(Option<u64>),
    /// Unsigned big integer with optional display width e.g. BIGINT UNSIGNED or BIGINT(20) UNSIGNED
    UnsignedBigInt(Option<u64>),
    /// Floating point e.g. REAL
    Real,
    /// Double e.g. DOUBLE PRECISION
    Double,
    /// Boolean
    Boolean,
    /// Date
    Date,
    /// Time
    Time,
    /// Timestamp without tz
    Timestamp(Option<u64>),
    /// DateTime without tz
    DateTime(Option<u64>),
    /// Interval
    Interval,
    /// Regclass used in postgresql serial
    Regclass,
    /// Text
    Text,
    /// String
    String,
    /// Bytea
    Bytea,
    /// Custom type such as enums
    Custom(ObjectName),
    /// Arrays
    Array(Box<DataType>),
    /// Struct
    Struct(Vec<Ident>, Vec<Box<DataType>>),
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Char(size) => format_type_with_optional_length(f, "CHAR", size, false),
            DataType::Varchar(size) => {
                format_type_with_optional_length(f, "CHARACTER VARYING", size, false)
            }
            DataType::Uuid => write!(f, "UUID"),
            DataType::Clob(size) => write!(f, "CLOB({})", size),
            DataType::Binary(size) => write!(f, "BINARY({})", size),
            DataType::Varbinary(size) => write!(f, "VARBINARY({})", size),
            DataType::Blob(size) => write!(f, "BLOB({})", size),
            DataType::Decimal(precision, scale) => {
                if let Some(scale) = scale {
                    write!(f, "NUMERIC({},{})", precision.unwrap(), scale)
                } else {
                    format_type_with_optional_length(f, "NUMERIC", precision, false)
                }
            }
            DataType::Float(size) => format_type_with_optional_length(f, "FLOAT", size, false),
            DataType::TinyInt(zerofill) => {
                format_type_with_optional_length(f, "TINYINT", zerofill, false)
            }
            DataType::UnsignedTinyInt(zerofill) => {
                format_type_with_optional_length(f, "TINYINT", zerofill, true)
            }
            DataType::SmallInt(zerofill) => {
                format_type_with_optional_length(f, "SMALLINT", zerofill, false)
            }
            DataType::UnsignedSmallInt(zerofill) => {
                format_type_with_optional_length(f, "SMALLINT", zerofill, true)
            }
            DataType::Int(zerofill) => format_type_with_optional_length(f, "INT", zerofill, false),
            DataType::UnsignedInt(zerofill) => {
                format_type_with_optional_length(f, "INT", zerofill, true)
            }
            DataType::BigInt(zerofill) => {
                format_type_with_optional_length(f, "BIGINT", zerofill, false)
            }
            DataType::UnsignedBigInt(zerofill) => {
                format_type_with_optional_length(f, "BIGINT", zerofill, true)
            }
            DataType::Real => write!(f, "REAL"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Date => write!(f, "DATE"),
            DataType::Time => write!(f, "TIME"),
            DataType::Timestamp(n) => format_type_with_optional_length(f, "TIMESTAMP", n, false),
            DataType::Interval => write!(f, "INTERVAL"),
            DataType::Regclass => write!(f, "REGCLASS"),
            DataType::Text => write!(f, "TEXT"),
            DataType::String => write!(f, "STRING"),
            DataType::Bytea => write!(f, "BYTEA"),
            DataType::Array(ty) => write!(f, "ARRAY({})", ty),
            DataType::Custom(ty) => write!(f, "{}", ty),
            DataType::DateTime(n) => format_type_with_optional_length(f, "DATETIME", n, false),
            DataType::Struct(names, types) => format_struct(f, names, types),
        }
    }
}

fn format_type_with_optional_length(
    f: &mut fmt::Formatter,
    sql_type: &'static str,
    len: &Option<u64>,
    unsigned: bool,
) -> fmt::Result {
    write!(f, "{}", sql_type)?;
    if let Some(len) = len {
        write!(f, "({})", len)?;
    }
    if unsigned {
        write!(f, " UNSIGNED")?;
    }
    Ok(())
}

fn format_struct(f: &mut fmt::Formatter, names: &[Ident], types: &[Box<DataType>]) -> fmt::Result {
    debug_assert!(names.len() == types.len());
    write!(f, "STRUCT(")?;
    let mut first = true;
    for (name, ty) in names.iter().zip(types.iter()) {
        if !first {
            write!(f, ", ")?;
        }
        first = false;
        write!(f, "{} {}", name, ty)?;
    }
    write!(f, ")")?;
    Ok(())
}
