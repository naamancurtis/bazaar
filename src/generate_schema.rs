use bazar::generate_schema;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let schema = generate_schema(None);
    let schema = schema.sdl();
    let mut file = File::create("schema.graphql")?;
    file.write_all(schema.as_bytes())?;
    Ok(())
}
