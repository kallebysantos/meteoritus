use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    response::{self, Responder},
    Request, Response, State,
};
use std::io::Cursor;
use uuid::Uuid;

use crate::Meteoritus;

#[post("/")]
pub fn creation_handler(_r: CreationRequest, meteoritus: &State<Meteoritus>) -> CreationResponder {
    // do resource creation

    let id = Uuid::new_v4().simple();

    let uri = format!("/files/{}", id);

    if let Some(callback) = &meteoritus.on_creation {
        callback();
    }

    CreationResponder::Success(uri)
}

pub enum CreationResponder {
    Success(String),
    Failure(Status, &'static str),
}

impl<'r> Responder<'r, 'static> for CreationResponder {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        match self {
            Self::Failure(status, error) => rocket::Response::build()
                .status(status)
                .sized_body(error.len(), Cursor::new(error))
                .ok(),

            Self::Success(uri) => Response::build()
                .header(Meteoritus::get_protocol_resumable_version())
                .raw_header("Location", uri)
                .status(Status::Created)
                .ok(),
        }
    }
}

#[derive(Debug)]
pub struct CreationRequest<'a> {
    content_length: u64,
    upload_length: u64,
    metadata: Option<&'a str>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CreationRequest<'r> {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let meteoritus = req.rocket().state::<Meteoritus>().unwrap();

        let tus_resumable_header = req.headers().get_one("Tus-Resumable");
        if tus_resumable_header.is_none() || tus_resumable_header.unwrap() != "1.0.0" {
            return Outcome::Failure((
                Status::BadRequest,
                "Missing or invalid Tus-Resumable header",
            ));
        }

        let content_length = match req.headers().get_one("Content-Length") {
            Some(value) => value.parse().unwrap_or(0),
            None => return Outcome::Failure((Status::BadRequest, "Missing Content-Length header")),
        };

        let upload_length = match req.headers().get_one("Upload-Length") {
            Some(value) => match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => {
                    return Outcome::Failure((Status::BadRequest, "Invalid Upload-Length header"))
                }
            },
            None => return Outcome::Failure((Status::BadRequest, "Missing Upload-Length header")),
        };

        if upload_length > meteoritus.max_size.as_u64() {
            return Outcome::Failure((
                Status::PayloadTooLarge,
                "Upload-Length exceeds the Tus-Max-Size",
            ));
        }

        let metadata = req.headers().get_one("Upload-Metadata");

        let creation_values = CreationRequest {
            content_length,
            upload_length,
            metadata,
        };

        Outcome::Success(creation_values)
    }
}
