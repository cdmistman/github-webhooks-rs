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
	#[error("no payload provided")]
	NoPayload,
	#[error("no secret defined")]
	NoSecret,
	#[error("no signature header")]
	NoSignature,
	#[error("invalid payload")]
	InvalidPayload(#[from] Rc<serde_json::Error>),
	#[error("unexpected error")]
	Other(#[from] Rc<dyn std::error::Error>),
}
