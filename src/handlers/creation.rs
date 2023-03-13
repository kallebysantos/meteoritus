use rocket::{
    http::{
        uri::{Origin, Reference},
        Status,
    },
    request::{self, FromRequest, Outcome},
    response::{self, Responder},
    Orbit, Request, Response, Rocket, State,
};
use std::{collections::HashMap, io::Cursor};

use crate::handlers::upload::*;
use crate::{comet_vault::CometFile, meteoritus::Meteoritus};

#[post("/")]
pub fn creation_handler(
    req: CreationRequest,
    meteoritus: &State<Meteoritus<Orbit>>,
) -> CreationResponder {
    let mut file = CometFile::new(req.upload_length).with_uuid();

    if let Some(metadata) = req.metadata {
        file.with_metadata(metadata);
    }

    let base = match Origin::parse(meteoritus.base_route()) {
        Ok(base) => base,
        Err(_) => {
            return CreationResponder::Failure(Status::InternalServerError, "some error");
        }
    };

    let uri = uri!(base, upload_handler(id = file.id()));
    let mut uri: Reference = uri.into();

    if let Some(callback) = &meteoritus.on_creation() {
        if let Err(error) = callback(req.rocket, &file, &mut uri) {
            return CreationResponder::Failure(Status::UnprocessableEntity, error);
        }
    }

    if let Err(_) = meteoritus.vault().add(&file) {
        return CreationResponder::Failure(Status::InternalServerError, "some error");
    };

    CreationResponder::Success(uri.to_string())
}

#[derive(Debug)]
pub struct CreationRequest<'r> {
    upload_length: u64,
    rocket: &'r Rocket<Orbit>,
    metadata: Option<HashMap<String, String>>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CreationRequest<'r> {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let meteoritus = req.rocket().state::<Meteoritus<Orbit>>().unwrap();

        let tus_resumable_header = req.headers().get_one("Tus-Resumable");
        if tus_resumable_header.is_none() || tus_resumable_header.unwrap() != "1.0.0" {
            return Outcome::Failure((
                Status::BadRequest,
                "Missing or invalid Tus-Resumable header",
            ));
        }

        let upload_length = match req.headers().get_one("Upload-Length") {
            Some(value) => match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => {
                    return Outcome::Failure((Status::BadRequest, "Invalid Upload-Length header"))
                }
            },
            None => return Outcome::Failure((Status::BadRequest, "Missing Upload-Length header")),
        };

        if upload_length > meteoritus.max_size().as_u64() {
            return Outcome::Failure((
                Status::PayloadTooLarge,
                "Upload-Length exceeds the Tus-Max-Size",
            ));
        }

        let metadata = match req.headers().get_one("Upload-Metadata") {
            None => None,
            Some(metadata) if metadata.is_empty() => None,
            Some(metadata) => Some(parse_tus_metadata(metadata)),
        };

        let creation_values = CreationRequest {
            upload_length,
            metadata,
            rocket: req.rocket(),
        };

        Outcome::Success(creation_values)
    }
}

pub enum CreationResponder {
    Success(String),
    Failure(Status, &'static str),
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

fn parse_tus_metadata(metadata_str: &str) -> HashMap<String, String> {
    let mut metadata_map = HashMap::new();

    if !metadata_str.is_empty() {
        for metadata_pair in metadata_str.split(',') {
            if let Some(idx) = metadata_pair.find(' ') {
                let (key, value) = metadata_pair.split_at(idx);
                let key = key.trim().to_string();
                let value = value.trim().to_string();

                metadata_map.insert(key, value);
            }
        }
    }

    metadata_map
}
