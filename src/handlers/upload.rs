use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    request::{self, FromRequest, Outcome},
    response::{self, Responder},
    Orbit, Request, Response, Rocket, State,
};

use crate::Meteoritus;

#[patch("/<id>")]
pub fn upload_handler(
    req: UploadRequest,
    id: &str,
    meteoritus: &State<Meteoritus>,
) -> UploadResponder {
    let is_upload_complete = true;

    if is_upload_complete {
        if let Some(callback) = &meteoritus.on_complete {
            callback(req.rocket);
        };
        return UploadResponder::Success();
    }

    // do patch update
    println!("Patching {}", id);

    UploadResponder::Success()
}

#[derive(Debug)]
pub struct UploadRequest<'r> {
    rocket: &'r Rocket<Orbit>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UploadRequest<'r> {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let _meteoritus = req.rocket().state::<Meteoritus>().unwrap();

        let tus_resumable_header = req.headers().get_one("Tus-Resumable");
        if tus_resumable_header.is_none() || tus_resumable_header.unwrap() != "1.0.0" {
            return Outcome::Failure((
                Status::BadRequest,
                "Missing or invalid Tus-Resumable header",
            ));
        }

        let _upload_offset = match req.headers().get_one("Upload-Offset") {
            Some(value) => match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => {
                    return Outcome::Failure((Status::BadRequest, "Invalid Upload-Offset header"))
                }
            },
            None => return Outcome::Failure((Status::BadRequest, "Missing Upload-Offset header")),
        };

        match req.content_type() {
            None => return Outcome::Failure((Status::BadRequest, "Missing Content-Type header")),
            Some(value) if value != &ContentType::new("application", "offset+octet-stream") => {
                return Outcome::Failure((
                    Status::UnsupportedMediaType,
                    "Invalid Content-Type header",
                ))
            }
            Some(_) => (),
        };

        let upload_values = UploadRequest {
            rocket: req.rocket(),
        };

        Outcome::Success(upload_values)
    }
}

pub enum UploadResponder {
    Success(),
}

impl<'r> Responder<'r, 'static> for UploadResponder {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        match self {

            Self::Success() => Response::build()
                .header(Meteoritus::get_protocol_resumable_version())
                .status(Status::Created)
                .ok(),
        }
    }
}
