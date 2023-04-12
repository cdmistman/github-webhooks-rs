//! # ocho-gato
//!
//! This crate provides a simple way to verify and deserialize webhook payloads
//! from GitHub. It provides the webhook schema under [`schema`], generated
//! using [`typify`](https://github.com/oxidecomputer/typify).
//!
//! ## Web Framework integrations
//!
//! This crate provides integrations for use in web frameworks. Currently, the
//! only framework is [`actix-web`](https://github.com/actix/actix-web), but
//! more may be added in the future.
//!
//! ### actix-web
//!
//! The actix-web integration is provided by the `actix-web` feature. An example
//! of using it is shown below.
//! ```rust,no_run
//! use actix_web::web;
//! use actix_web::App;
//! use actix_web::HttpResponse;
//! use actix_web::HttpServer;
//! use actix_web::Responder;
//! use ocho_gato::schema::WebhookPayload;
//! use ocho_gato::Secret;
//!
//! #[actix_web::post("/webhook")]
//! async fn webhook(payload: WebhookPayload) -> impl Responder {
//! 	HttpResponse::Ok().body(format!("Received webhook: {:?}", payload))
//! }
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//! 	HttpServer::new(|| App::new().service(webhook))
//! 		.on_connect(|_, exts| {
//! 			exts.insert(Secret(
//! 				std::env::var("GITHUB_WEBHOOK_SECRET")
//! 					.unwrap_or_default()
//! 					.into(),
//! 			));
//! 		})
//! 		.bind(("0.0.0.0", 8080))?
//! 		.run()
//! 		.await
//! }
//! ```

use std::rc::Rc;

#[macro_use]
extern crate serde;

#[cfg(feature = "actix-web")]
mod actix_impl;
pub mod schema;

#[derive(Clone, Debug)]
pub struct Secret(pub Rc<str>);

#[derive(Clone, Debug, thiserror::Error)]
pub enum PayloadError {
	#[error("invalid payload: {0}")]
	InvalidPayload(#[from] Rc<serde_json::Error>),
	#[error("invalid signature")]
	InvalidSignature,

	#[error("no payload provided")]
	NoPayload,
	#[error("no secret defined")]
	NoSecret,
	#[error("no signature header")]
	NoSignature,

	#[error("internal error")]
	Other(#[from] Rc<dyn std::error::Error>),
}
