//! # ocho-gato
//!
//! This crate provides a simple way to verify and deserialize webhook payloads
//! from GitHub. It provides the webhook schema under [`schema`], generated
//! using [`typify`](https://github.com/oxidecomputer/typify).

#[macro_use]
extern crate serde;

mod schema;

use hmac::Hmac;
use hmac::Mac;
use sha2::Sha256;

pub use self::schema::*;

#[derive(Debug, thiserror::Error)]
pub enum VerifiedParseError {
	#[error("invalid payload")]
	InvalidPayload,
	#[error("invalid secret")]
	InvalidSecret,
	#[error("invalid signature")]
	InvalidSignature,
}

impl WebhookPayload {
	pub fn parse_verified(
		body: &impl AsRef<[u8]>,
		sig: &impl AsRef<[u8]>,
		secret: &impl AsRef<[u8]>,
	) -> Result<Self, VerifiedParseError> {
		let body = body.as_ref();
		let sig = sig.as_ref();
		let secret = secret.as_ref();

		let mut hasher = Hmac::<Sha256>::new_from_slice(secret)
			.map_err(|_| VerifiedParseError::InvalidSecret)?;
		hasher.update(body);
		let hash = hex::encode(hasher.finalize().into_bytes()).into_bytes();

		if &*hash != sig {
			return Err(VerifiedParseError::InvalidSignature);
		}

		serde_json::from_slice(secret).map_err(|_| VerifiedParseError::InvalidPayload)
	}
}
