use rocket::{
    http::{
        uri::{Origin, Reference},
        Status,
    },
    request::{self, FromRequest, Outcome},
    response::{self, Responder},
    Orbit, Request, Response, Rocket, State,
};
use std::{io::Cursor, sync::Arc};

use crate::meteoritus::Meteoritus;
use crate::{handlers::upload::*, Vault};

use super::HandlerContext;

#[post("/")]
pub fn creation_handler(
    req: CreationRequest,
    meteoritus: &State<Meteoritus<Orbit>>,
    vault: &State<Arc<dyn Vault>>,
) -> CreationResponder {
    let file = match vault.build_file(req.upload_length, req.metadata) {
        Ok(file) => file,
        Err(_) => {
            return CreationResponder::Failure(
                Status::InternalServerError,
                "creation error".to_string(),
            )
        }
    };

    let base_uri = match Origin::parse(meteoritus.base_route()) {
        Ok(base) => base,
        Err(_) => {
            return CreationResponder::Failure(
                Status::InternalServerError,
                "some error".to_string(),
            );
        }
    };

    let uri = uri!(base_uri, upload_handler(id = file.id()));
    let uri: Reference = uri.into();

    if let Some(callback) = &meteoritus.on_creation() {
        if let Err(error) = callback(HandlerContext {
            rocket: req.rocket,
            file_info: &file,
        }) {
            return CreationResponder::Failure(
                Status::UnprocessableEntity,
                error.to_string(),
            );
        }
    }

    match vault.create_file(file) {
        Ok(file) => {
            if let Some(callback) = &meteoritus.on_created() {
                callback(HandlerContext {
                    rocket: req.rocket,
                    file_info: &file,
                });
            }

            CreationResponder::Success(uri.to_string())
        }
        Err(_) => CreationResponder::Failure(
            Status::InternalServerError,
            "some vault error".to_string(),
        ),
    }
}

#[derive(Debug)]
pub struct CreationRequest<'r> {
    rocket: &'r Rocket<Orbit>,
    upload_length: u64,
    metadata: Option<&'r str>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CreationRequest<'r> {
    type Error = &'static str;

    async fn from_request(
        req: &'r Request<'_>,
    ) -> request::Outcome<Self, Self::Error> {
        let meteoritus = req.rocket().state::<Meteoritus<Orbit>>().unwrap();

        let tus_resumable_header = req.headers().get_one("Tus-Resumable");
        if tus_resumable_header.is_none()
            || tus_resumable_header.unwrap() != "1.0.0"
        {
            return Outcome::Error((
                Status::BadRequest,
                "Missing or invalid Tus-Resumable header",
            ));
        }

        let upload_length = match req.headers().get_one("Upload-Length") {
            Some(value) => match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => {
                    return Outcome::Error((
                        Status::BadRequest,
                        "Invalid Upload-Length header",
                    ))
                }
            },
            None => {
                return Outcome::Error((
                    Status::BadRequest,
                    "Missing Upload-Length header",
                ))
            }
        };

        if upload_length > meteoritus.max_size().as_u64() {
            return Outcome::Error((
                Status::PayloadTooLarge,
                "Upload-Length exceeds the Tus-Max-Size",
            ));
        }

        let metadata = match req.headers().get_one("Upload-Metadata") {
            None => None,
            Some(metadata) if metadata.is_empty() => None,
            Some(metadata) => Some(metadata),
        };

        let creation_values = CreationRequest {
            rocket: req.rocket(),
            upload_length,
            metadata,
        };

        Outcome::Success(creation_values)
    }
}

pub enum CreationResponder {
    Success(String),
    Failure(Status, String),
}

impl<'r> Responder<'r, 'static> for CreationResponder {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let meteoritus = req.rocket().state::<Meteoritus<Orbit>>().unwrap();

        match self {
            Self::Failure(status, error) => rocket::Response::build()
                .status(status)
                .sized_body(error.len(), Cursor::new(error))
                .ok(),

            Self::Success(uri) => Response::build()
                .header(meteoritus.get_protocol_resumable_version())
                .raw_header("Location", uri)
                .status(Status::Created)
                .ok(),
        }
    }
}
