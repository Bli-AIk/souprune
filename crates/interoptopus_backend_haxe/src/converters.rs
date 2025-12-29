//! Type converters from Interoptopus types to Haxe types.

use crate::interop::Interop;
use heck::ToLowerCamelCase;
use interoptopus::lang::{
    Composite, Constant, ConstantValue, Enum, Function, Opaque, Primitive, PrimitiveValue, Type,
    Variant,
};
use interoptopus::pattern::TypePattern;

/// Convert Rust primitive to Haxe type name.
pub fn primitive_to_haxe(x: Primitive) -> &'static str {
    match x {
        Primitive::Void => "Void",
        Primitive::Bool => "Bool",
        Primitive::U8 => "cpp.UInt8",
        Primitive::U16 => "cpp.UInt16",
        Primitive::U32 => "cpp.UInt32",
        Primitive::U64 => "cpp.UInt64",
        Primitive::Usize => "cpp.SizeT",
        Primitive::I8 => "cpp.Int8",
        Primitive::I16 => "cpp.Int16",
        Primitive::I32 => "Int",
        Primitive::I64 => "cpp.Int64",
        Primitive::Isize => "cpp.SizeT",
        Primitive::F32 => "cpp.Float32",
        Primitive::F64 => "Float",
    }
}

/// Convert enum type to Haxe type name.
pub fn enum_to_typename(_g: &Interop, x: &Enum) -> String {
    x.rust_name().to_string()
}

/// Convert enum variant to Haxe enum value name.
pub fn enum_variant_to_name(_g: &Interop, _the_enum: &Enum, x: &Variant) -> String {
    x.name().to_string()
}

/// Convert opaque type to Haxe type name.
pub fn opaque_to_typename(_g: &Interop, x: &Opaque) -> String {
    x.rust_name().to_string()
}

/// Convert composite (struct) to Haxe type name.
pub fn composite_to_typename(_g: &Interop, x: &Composite) -> String {
    x.rust_name().to_string()
}

/// Convert any type to Haxe type specifier.
pub fn to_haxe_type(g: &Interop, x: &Type) -> String {
    match x {
        Type::Primitive(p) => primitive_to_haxe(*p).to_string(),
        Type::Enum(e) => enum_to_typename(g, e),
        Type::Opaque(o) => format!("cpp.Pointer<{}>", opaque_to_typename(g, o)),
        Type::Composite(c) => composite_to_typename(g, c),
        Type::Wire(_) => "cpp.RawPointer<cpp.Void>".to_string(),
        Type::WirePayload(_) => "cpp.RawPointer<cpp.Void>".to_string(),
        Type::ReadPointer(inner) => format!("cpp.ConstPointer<{}>", to_haxe_type(g, inner)),
        Type::ReadWritePointer(inner) => format!("cpp.Pointer<{}>", to_haxe_type(g, inner)),
        Type::FnPointer(_) => "cpp.RawPointer<cpp.Void>".to_string(), // Function pointers need special handling
        Type::Pattern(TypePattern::CChar) => "cpp.Char".to_string(),
        Type::Pattern(TypePattern::CStrPointer) => "cpp.ConstCharStar".to_string(),
        Type::Pattern(p) => to_haxe_type(g, &p.fallback_type()),
        Type::Array(_) => "cpp.RawPointer<cpp.Void>".to_string(), // Arrays need special handling
    }
}

/// Convert function name to Haxe method name (camelCase).
pub fn function_name_to_haxe(function: &Function) -> String {
    function.name().to_lower_camel_case()
}

/// Convert function name to C symbol name (unchanged).
pub fn function_name_to_c_name(function: &Function) -> String {
    function.name().to_string()
}

/// Convert constant to Haxe constant name.
pub fn const_name_to_haxe(_g: &Interop, x: &Constant) -> String {
    x.name().to_string()
}

/// Convert constant value to Haxe literal.
pub fn constant_value_to_haxe(value: &ConstantValue) -> String {
    match value {
        ConstantValue::Primitive(x) => match x {
            PrimitiveValue::Bool(v) => if *v { "true" } else { "false" }.to_string(),
            PrimitiveValue::U8(v) => format!("{v}"),
            PrimitiveValue::U16(v) => format!("{v}"),
            PrimitiveValue::U32(v) => format!("{v}"),
            PrimitiveValue::U64(v) => format!("{v}"),
            PrimitiveValue::Usize(v) => format!("{v}"),
            PrimitiveValue::I8(v) => format!("{v}"),
            PrimitiveValue::I16(v) => format!("{v}"),
            PrimitiveValue::I32(v) => format!("{v}"),
            PrimitiveValue::I64(v) => format!("{v}"),
            PrimitiveValue::Isize(v) => format!("{v}"),
            PrimitiveValue::F32(v) => format!("{v}"),
            PrimitiveValue::F64(v) => format!("{v}"),
        },
    }
}
