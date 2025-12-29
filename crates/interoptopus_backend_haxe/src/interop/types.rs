//! Type definition generation for Haxe.

use crate::converters::{enum_to_typename, enum_variant_to_name};
use crate::interop::Interop;
use interoptopus::lang::util::sort_types_by_dependencies;
use interoptopus::lang::{Enum, Type, VariantKind};
use interoptopus_backend_utils::{Error, IndentWriter};

/// Write all type definitions to the Haxe output.
pub fn write_type_definitions(g: &Interop, w: &mut IndentWriter) -> Result<(), Error> {
    let types = sort_types_by_dependencies(g.inventory().c_types().to_vec());

    // Enums are written inline in the extern class
    // Composites (structs) are written separately

    let enums: Vec<_> = types
        .iter()
        .filter_map(|t| if let Type::Enum(e) = t { Some(e) } else { None })
        .collect();

    if !enums.is_empty() {
        w.indented(|w| writeln!(w, "// Enum type aliases (represented as Int in hxcpp)"))?;
        for the_enum in enums {
            write_enum_typedef(g, w, the_enum)?;
        }
        w.newline()?;
    }

    Ok(())
}

fn write_enum_typedef(g: &Interop, w: &mut IndentWriter, the_enum: &Enum) -> Result<(), Error> {
    let enum_name = enum_to_typename(g, the_enum);

    // Write documentation
    let doc_lines = the_enum.meta().docs().lines();
    if let Some(doc) = doc_lines.first() {
        w.indented(|w| writeln!(w, "/** {} */", doc))?;
    }

    // In hxcpp, C enums are typically represented as Int
    // We generate a type alias and constants for the values
    w.indented(|w| writeln!(w, "// {} enum values", enum_name))?;

    for variant in the_enum.variants() {
        let variant_name = enum_variant_to_name(g, the_enum, variant);

        match variant.kind() {
            VariantKind::Unit(value) => {
                w.indented(|w| {
                    writeln!(
                        w,
                        "public static inline var {}_{}: Int = {};",
                        enum_name.to_uppercase(),
                        variant_name.to_uppercase(),
                        value
                    )
                })?;
            }
            VariantKind::Typed(_, _) => {
                // Complex enum variants are not supported in C ABI
                w.indented(|w| writeln!(w, "// Skipped: {} (typed variant)", variant_name))?;
            }
        }
    }

    Ok(())
}
