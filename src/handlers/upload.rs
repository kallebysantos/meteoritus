use rocket::{
    data::Capped,
    http::{ContentType, Status},
    request::{self, FromRequest, Outcome},
    response::{self, Responder},
    Orbit, Request, Rocket, State,
};

use crate::Meteoritus;

#[patch("/<id>", data = "<data>")]
pub fn upload_handler(
    req: UploadRequest,
    id: &str,
    meteoritus: &State<Meteoritus>,
    mut data: Capped<Vec<u8>>,
) -> UploadResponder {
    let mut file = match meteoritus.vault.take(id.to_string()) {
        Ok(file) => file,
        Err(_) => return UploadResponder::Failure(Status::NotFound),
    };

    if file.offset() != &req.upload_offset {
        return UploadResponder::Failure(Status::Conflict);
    }

    if let Err(_) = file.set_offset(req.upload_offset) {
        return UploadResponder::Failure(Status::PayloadTooLarge);
    };

    if let Err(_) = meteoritus.vault.update(&mut file, &mut data) {
        return UploadResponder::Failure(Status::UnprocessableEntity);
    };

    let is_upload_complete = file.length() == file.offset();

    if is_upload_complete {
        if let Some(callback) = &meteoritus.on_complete {
            callback(req.rocket);
        };
        return UploadResponder::Success(*file.offset());
    }

    UploadResponder::Success(*file.offset())
}

#[derive(Debug)]
pub struct UploadRequest<'r> {
    rocket: &'r Rocket<Orbit>,
    upload_offset: u64,
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

        let upload_offset = match req.headers().get_one("Upload-Offset") {
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
            upload_offset,
        };

        Outcome::Success(upload_values)
    }
}

pub enum UploadResponder {
    Success(u64),
    Failure(Status),
}

impl<'r> Responder<'r, 'static> for UploadResponder {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let mut res = rocket::Response::build();

        res.header(Meteoritus::get_protocol_resumable_version());

        match self {
            Self::Success(offset) => {
                res.status(Status::NoContent);
                res.raw_header("Upload-Offset", offset.to_string())
            }
            Self::Failure(status) => res.status(status),
        };

        res.ok()
    }
}
