use std::fs::OpenOptions;
use std::io::Write;

use reqwest::blocking::Client;
use reqwest::redirect;
use schemars::schema::RootSchema;
use typify::TypeSpace;

const SCHEMA_PATH: &str = "src/schema.rs";

fn main() -> eyre::Result<()> {
	let mut type_space = TypeSpace::default();

	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(SCHEMA_PATH)?;

	let schema_text = Client::builder()
		.redirect(redirect::Policy::custom(policy))
		.build()?
		.get("https://unpkg.com/@octokit/webhooks-schemas/schema.json")
		.send()?
		.text()?;

	let mut schema: RootSchema = serde_json::from_str(&schema_text)?;
	schema.schema.metadata().title = Some("WebhookSchema".to_string());
	type_space
		.add_root_schema(schema)?
		.ok_or_else(|| eyre::eyre!("expected schema to be accepted"))?;

	file.write_all(type_space.to_stream().to_string().as_bytes())?;

	Ok(())
}

fn policy(attempt: redirect::Attempt) -> redirect::Action {
	let url = attempt.url();

	if url.host_str() != Some("unpkg.com") {
		return attempt.error(eyre::eyre!("expected redirect to stay in unpkg.com"));
	}

	let path = url
		.path_segments()
		.ok_or_else(|| eyre::eyre!("no path segments for redirect: {attempt:?}"));
	let path = match path {
		Ok(path_segments) => path_segments.collect::<Vec<&str>>(),
		Err(error) => return attempt.error(error),
	};

	if path.len() != 3 {
		let error = eyre::eyre!("unexpected redirect path: {attempt:?}");
		return attempt.error(error);
	}

	let (org, repo, file) = (path[0], path[1], path[2]);
	if org != "@octokit" {
		let error = eyre::eyre!("expected octokit organization: {attempt:?}");
		return attempt.error(error);
	}

	if !repo.starts_with("webhooks-schemas@") {
		let error = eyre::eyre!("expected valid repository and version: {attempt:?}");
		return attempt.error(error);
	}

	if file != "schema.json" {
		let error = eyre::eyre!("expected schema.json file: {attempt:?}");
		return attempt.error(error);
	}

	attempt.follow()
}
