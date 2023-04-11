mod actix {
	pub use actix_web::body::BoxBody;
	pub use actix_web::dev::Payload;
	pub use actix_web::http::StatusCode;
	pub use actix_web::web::Bytes;
	pub use actix_web::FromRequest;
	pub use actix_web::HttpRequest;
	pub use actix_web::HttpResponse;
	pub use actix_web::ResponseError;
}

use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::ready;
use std::task::Context;
use std::task::Poll;

use hmac::Hmac;
use hmac::Mac;
use sha2::Sha256;

use crate::schema::WebhookPayload;
use crate::PayloadError;
use crate::Secret;

impl actix::ResponseError for PayloadError {
	fn error_response(&self) -> actix::HttpResponse<actix::BoxBody> {
		todo!()
	}

	fn status_code(&self) -> actix::StatusCode {
		todo!()
	}
}

impl actix::FromRequest for WebhookPayload {
	type Error = PayloadError;
	// TODO: return proper future to prevent timing attacks
	// maybe require a start time to be passed via conn_data
	type Future = WebhookPayloadExtractorFut;

	fn extract<'r>(_: &'r actix::HttpRequest) -> Self::Future {
		WebhookPayloadExtractorFut(Err(PayloadError::NoPayload))
	}

	fn from_request(req: &actix::HttpRequest, payload: &mut actix::Payload) -> Self::Future {
		const SIGNATURE_START: &str = "sha256=";

		let Some(secret) = req.conn_data::<Secret>() else {
			return WebhookPayloadExtractorFut(Err(PayloadError::NoSecret));
		};

		let Some(signature_header) = req.headers().get("x-hub-signature-256") else {
			// signature not provided
			return WebhookPayloadExtractorFut(Err(PayloadError::NoSignature));
		};
		let Ok(signature) = signature_header.to_str() else {
			// signature not valid utf8
			return WebhookPayloadExtractorFut(Err(PayloadError::NoSignature));
		};
		if !signature.starts_with(SIGNATURE_START) {
			// signature not valid sha256 hash
			return WebhookPayloadExtractorFut(Err(PayloadError::NoSignature));
		};
		let signature = &signature[SIGNATURE_START.len()..];

		WebhookPayloadExtractorFut(Ok(WebhookPayloadExtractor {
			secret:    secret.clone(),
			signature: signature.to_string(),
			payload:   actix::Bytes::from_request(req, payload),
		}))
	}
}

#[doc(hidden)]
pub struct WebhookPayloadExtractorFut(Result<WebhookPayloadExtractor, PayloadError>);

struct WebhookPayloadExtractor {
	secret:    crate::Secret,
	signature: String,
	payload:   <actix::Bytes as actix::FromRequest>::Future,
}

impl Future for WebhookPayloadExtractorFut {
	type Output = Result<WebhookPayload, PayloadError>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		match self.0.as_mut() {
			Err(e) => Poll::Ready(Err(e.clone())),
			Ok(extractor) => {
				let mut extractor = Pin::new(extractor);
				let payload = match ready!(Pin::new(&mut extractor.payload).poll(cx)) {
					Ok(payload) => payload,
					Err(e) => return Poll::Ready(Err(PayloadError::Other(Rc::new(e)))),
				};
				let mut hasher =
					Hmac::<Sha256>::new_from_slice(&extractor.secret.0.as_bytes()).unwrap();
				hasher.update(&payload);
				let hash = hasher.finalize().into_bytes();
				let hash = hex::encode(hash);
				if hash != extractor.signature {
					return Poll::Ready(Err(PayloadError::NoSignature));
				}
				let payload = serde_json::from_slice(&payload).map_err(Rc::new)?;
				Poll::Ready(Ok(payload))
			},
		}
	}
}
