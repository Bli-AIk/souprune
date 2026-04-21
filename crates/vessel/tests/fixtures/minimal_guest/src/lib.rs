use anyhow::Result;
use serde::Serialize;
use souprune_vessel::prelude::*;

#[derive(Serialize)]
struct ExampleConfig {
    name: String,
    count: u32,
}

vessel_guest! {
    fn build(reg: &mut Registry) -> Result<()> {
        reg.emit_ron(
            "example/test.ron",
            &ExampleConfig {
                name: "fixture".to_string(),
                count: 3,
            },
        )?;
        Ok(())
    }
}
