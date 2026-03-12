use crate::function::ParameterAttribute;
use crate::types::{TypeRef, Typed, Types};
use crate::{ConstantRef, Name};
use std::fmt::{self, Display};

#[derive(PartialEq, Clone, Debug, Hash)]
pub enum Operand {
    /// e.g., `i32 %foo`
    LocalOperand {
        name: Name,
        ty: TypeRef,
    },
    /// includes [`GlobalReference`](../constant/enum.Constant.html#variant.GlobalReference) for things like `@foo`
    ConstantOperand(ConstantRef),
    MetadataOperand, // --TODO not yet implemented-- MetadataOperand(Box<Metadata>),
}

impl Typed for Operand {
    fn get_type(&self, types: &Types) -> TypeRef {
        match self {
            Operand::LocalOperand { ty, .. } => ty.clone(),
            Operand::ConstantOperand(c) => types.type_of(c),
            Operand::MetadataOperand => types.metadata_type(),
        }
    }
}

impl Operand {
    /// Get a reference to the `Constant`, if the operand is a constant;
    /// otherwise, returns `None`.
    ///
    /// This allows nested matching on `Operand`. Instead of the following code
    /// (which doesn't compile because you can't directly match on `ConstantRef`)
    /// ```ignore
    /// if let Operand::ConstantOperand(Constant::Float(Float::Double(val))) = op
    /// ```
    /// you can write this:
    /// ```ignore
    /// if let Some(Constant::Float(Float::Double(val))) = op.as_constant()
    /// ```
    pub fn as_constant(&self) -> Option<&Constant> {
        match self {
            Operand::ConstantOperand(cref) => Some(cref),
            _ => None,
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::LocalOperand { name, ty } => write!(f, "{} {}", ty, name),
            Operand::ConstantOperand(cref) => write!(f, "{}", &cref),
            Operand::MetadataOperand => write!(f, "<metadata>"),
        }
    }
}

/// Format just the value part of an operand (without the type prefix).
/// For example, a `LocalOperand { name: %3, ty: i32 }` would display as just `%3`
/// instead of `i32 %3`.
pub fn fmt_operand_value(op: &Operand, f: &mut fmt::Formatter) -> fmt::Result {
    match op {
        Operand::LocalOperand { name, .. } => write!(f, "{}", name),
        Operand::ConstantOperand(cref) => fmt_constant_value(cref, f),
        Operand::MetadataOperand => write!(f, "<metadata>"),
    }
}

/// Format just the value part of a constant (without its type prefix).
/// For constants like Int that display as "i32 5", this returns just "5".
/// For aggregate constants like Array/Struct/Vector, returns the value body
/// (including inner element types).
pub(crate) fn fmt_constant_value(cref: &ConstantRef, f: &mut fmt::Formatter) -> fmt::Result {
    use crate::constant::Constant;
    match cref.as_ref() {
        Constant::Int { bits, value } => {
            if *bits == 1 {
                if *value == 0 {
                    write!(f, "false")
                } else {
                    write!(f, "true")
                }
            } else {
                match *bits {
                    16 => {
                        let signed_val = (*value & 0xFFFF) as i16;
                        if signed_val > -1000 {
                            write!(f, "{}", signed_val)
                        } else {
                            write!(f, "{}", *value)
                        }
                    },
                    32 => {
                        let signed_val = (*value & 0xFFFF_FFFF) as i32;
                        if signed_val > -1000 {
                            write!(f, "{}", signed_val)
                        } else {
                            write!(f, "{}", *value)
                        }
                    },
                    64 => {
                        let signed_val = *value as i64;
                        if signed_val > -1000 {
                            write!(f, "{}", signed_val)
                        } else {
                            write!(f, "{}", *value)
                        }
                    },
                    _ => write!(f, "{}", *value),
                }
            }
        },
        Constant::Null(_) => write!(f, "null"),
        Constant::AggregateZero(_) => write!(f, "zeroinitializer"),
        Constant::Undef(_) => write!(f, "undef"),
        #[cfg(feature = "llvm-12-or-greater")]
        Constant::Poison(_) => write!(f, "poison"),
        Constant::GlobalReference { name, .. } => {
            match name {
                Name::Name(n) => write!(f, "@{}", n),
                Name::Number(n) => write!(f, "@{}", n),
            }
        },
        Constant::TokenNone => write!(f, "none"),
        // These already display without a type prefix — just the value body
        Constant::Array { .. } | Constant::Vector(..) | Constant::Struct { .. } => {
            write!(f, "{}", cref)
        },
        // Float Display includes the type name (e.g., "double 3.14"), extract just the value
        Constant::Float(float) => {
            use crate::constant::Float;
            match float {
                Float::Single(s) => write!(f, "{}", s),
                Float::Double(d) => write!(f, "{}", d),
                _ => write!(f, "{}", float), // other float types: fallback
            }
        },
        // For everything else (constant expressions), use full display
        _ => write!(f, "{}", cref),
    }
}

/// Format an operand with parameter attributes inserted between the type and value.
/// In LLVM IR syntax, call argument attributes go between type and value: `ptr nonnull %x`
pub fn fmt_operand_with_attrs(
    op: &Operand,
    attrs: &[ParameterAttribute],
    f: &mut fmt::Formatter,
) -> fmt::Result {
    if attrs.is_empty() {
        return write!(f, "{}", op);
    }
    match op {
        Operand::LocalOperand { name, ty } => {
            write!(f, "{}", ty)?;
            for attr in attrs {
                let s = attr.to_string();
                if !s.is_empty() {
                    write!(f, " {}", s)?;
                }
            }
            write!(f, " {}", name)
        },
        Operand::ConstantOperand(cref) => {
            use crate::constant::Constant;
            // For constants that display as "type value", insert attrs between type and value
            // We need to write the type, then attrs, then the value
            match cref.as_ref() {
                Constant::Int { bits, .. } => {
                    write!(f, "i{}", bits)?;
                    for attr in attrs {
                        let s = attr.to_string();
                        if !s.is_empty() {
                            write!(f, " {}", s)?;
                        }
                    }
                    write!(f, " ")?;
                    fmt_constant_value(cref, f)
                },
                Constant::GlobalReference { name, ty } => {
                    match ty.as_ref() {
                        crate::types::Type::FuncType { .. } => write!(f, "{}", ty)?,
                        _ => {
                            #[cfg(feature = "llvm-14-or-lower")]
                            write!(f, "{}*", ty)?;
                            #[cfg(feature = "llvm-15-or-greater")]
                            write!(f, "ptr")?;
                        },
                    }
                    for attr in attrs {
                        let s = attr.to_string();
                        if !s.is_empty() {
                            write!(f, " {}", s)?;
                        }
                    }
                    write!(f, " ")?;
                    match name {
                        Name::Name(n) => write!(f, "@{}", n),
                        Name::Number(n) => write!(f, "@{}", n),
                    }
                },
                Constant::Null(ty) => {
                    write!(f, "{}", ty)?;
                    for attr in attrs {
                        let s = attr.to_string();
                        if !s.is_empty() {
                            write!(f, " {}", s)?;
                        }
                    }
                    write!(f, " null")
                },
                // Fallback: write the full constant then attrs after
                _ => {
                    write!(f, "{}", cref)?;
                    for attr in attrs {
                        let s = attr.to_string();
                        if !s.is_empty() {
                            write!(f, " {}", s)?;
                        }
                    }
                    Ok(())
                },
            }
        },
        Operand::MetadataOperand => {
            write!(f, "<metadata>")?;
            for attr in attrs {
                let s = attr.to_string();
                if !s.is_empty() {
                    write!(f, " {}", s)?;
                }
            }
            Ok(())
        },
    }
}

// ********* //
// from_llvm //
// ********* //

use crate::constant::Constant;
use crate::function::FunctionContext;
use crate::llvm_sys::*;
use crate::module::ModuleContext;
use llvm_sys::LLVMValueKind;

impl Operand {
    pub(crate) fn from_llvm_ref(
        operand: LLVMValueRef,
        ctx: &mut ModuleContext,
        func_ctx: &FunctionContext,
    ) -> Self {
        let constant = unsafe { LLVMIsAConstant(operand) };
        if !constant.is_null() {
            Operand::ConstantOperand(Constant::from_llvm_ref(constant, ctx))
        } else if unsafe {
            LLVMGetValueKind(operand) == LLVMValueKind::LLVMMetadataAsValueValueKind
        } {
            Operand::MetadataOperand
        } else {
            Operand::LocalOperand {
                name: func_ctx.val_names
                    .get(&operand)
                    .unwrap_or_else(|| {
                        let names: Vec<_> = func_ctx.val_names.values().collect();
                        let kind = unsafe { LLVMGetValueKind(operand) };
                        panic!(
                            "Failed to find operand with kind {:?} in func_ctx.val_names; have names {:?}",
                            kind, names
                        )
                    })
                    .clone(),
                ty: ctx.types.type_from_llvm_ref(unsafe { LLVMTypeOf(operand) }),
            }
        }
    }
}
