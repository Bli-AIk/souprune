//! Constants generation for Haxe.

use crate::converters::{const_name_to_haxe, constant_value_to_haxe};
use crate::interop::Interop;
use interoptopus_backend_utils::{Error, IndentWriter};

/// Write all constants to the Haxe output.
pub fn write_constants(g: &Interop, w: &mut IndentWriter) -> Result<(), Error> {
    let constants = g.inventory().constants();

    if constants.is_empty() {
        return Ok(());
    }

    w.indented(|w| writeln!(w, "// Constants"))?;

    for constant in constants {
        let name = const_name_to_haxe(g, constant);
        let value = constant_value_to_haxe(constant.value());

        let doc_lines = constant.meta().docs().lines();
        if let Some(doc) = doc_lines.first() {
            w.indented(|w| writeln!(w, "/** {} */", doc))?;
        }

        w.indented(|w| {
            writeln!(
                w,
                "public static inline var {}:{} = {};",
                name,
                "Int", // TODO: determine type from constant
                value
            )
        })?;
    }

    w.newline()?;

    Ok(())
}
