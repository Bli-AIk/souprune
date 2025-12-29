//! Function generation for Haxe.

use crate::converters::{function_name_to_c_name, function_name_to_haxe, to_haxe_type};
use crate::interop::Interop;
use interoptopus_backend_utils::{Error, IndentWriter};

/// Write all function declarations to the Haxe output.
pub fn write_functions(g: &Interop, w: &mut IndentWriter) -> Result<(), Error> {
    let functions = g.inventory().functions();

    if functions.is_empty() {
        return Ok(());
    }

    w.indented(|w| writeln!(w, "// Functions"))?;

    for function in functions {
        write_function(g, w, function)?;
    }

    Ok(())
}

fn write_function(
    g: &Interop,
    w: &mut IndentWriter,
    function: &interoptopus::lang::Function,
) -> Result<(), Error> {
    let haxe_name = function_name_to_haxe(function);
    let c_name = function_name_to_c_name(function);
    let signature = function.signature();
    let return_type = to_haxe_type(g, signature.rval());

    // Build parameter list
    let params: Vec<String> = signature
        .params()
        .iter()
        .map(|p| {
            let param_name = p.name();
            let param_type = to_haxe_type(g, p.the_type());
            format!("{}:{}", param_name, param_type)
        })
        .collect();

    // Write documentation
    let doc_lines = function.meta().docs().lines();
    if let Some(doc) = doc_lines.first() {
        w.indented(|w| writeln!(w, "/** {} */", doc))?;
    }

    // Write the native annotation and function declaration
    w.indented(|w| writeln!(w, "@:native(\"{}\")", c_name))?;
    w.indented(|w| {
        writeln!(
            w,
            "public static function {}({}):{};",
            haxe_name,
            params.join(", "),
            return_type
        )
    })?;

    w.newline()?;

    Ok(())
}
