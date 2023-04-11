use std::fs::OpenOptions;
use std::io::Write;

use schemars::schema::RootSchema;
use typify::TypeSpace;

const CODEGEN_PATH: &str = "src/schema.rs";
const SCHEMA_PATH: &str = "schema.json";

fn main() -> eyre::Result<()> {
	if !std::path::Path::new(SCHEMA_PATH).exists() {
		return Ok(());
	}

	println!("cargo:rerun-if-changed={}", SCHEMA_PATH);

	let mut type_space = TypeSpace::default();

	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(CODEGEN_PATH)?;

	let schema_text = OpenOptions::new().read(true).open(SCHEMA_PATH)?;
	let mut schema: RootSchema = serde_json::from_reader(schema_text)?;

	schema.schema.metadata().title = Some("WebhookPayload".to_string());
	type_space
		.add_root_schema(schema)?
		.ok_or_else(|| eyre::eyre!("expected schema to be accepted"))?;

	file.write_all(type_space.to_stream().to_string().as_bytes())?;

	Ok(())
}
