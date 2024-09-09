use std::sync::Arc;

use rocket::{
    http::{ContentType, Status},
    request::{self, FromRequest, Outcome},
    response::{self, Responder},
    Data, Orbit, Request, Rocket, State,
};

use crate::{fs::PatchOption, Meteoritus, Vault};

use super::HandlerContext;

#[patch("/<id>", data = "<data>")]
pub async fn upload_handler(
    req: UploadRequest<'_>,
    id: &str,
    meteoritus: &State<Meteoritus<Orbit>>,
    data: Data<'_>,
    vault: &State<Arc<dyn Vault>>,
) -> UploadResponder {
    if !vault.exists(id) {
        return UploadResponder::Failure(Status::NotFound);
    }

    let Ok(mut data) = data.open(meteoritus.max_size()).into_bytes().await
    else {
        return UploadResponder::Failure(Status::UnprocessableEntity);
    };

    let Ok(result) = vault.patch_file(id, &mut data, req.offset) else {
        return UploadResponder::Failure(Status::UnprocessableEntity);
    };

    let final_offset = match result {
        PatchOption::Patched(offset) => offset,
        PatchOption::Completed(file) => {
            if let Some(callback) = &meteoritus.on_completed() {
                callback(HandlerContext {
                    rocket: req.rocket,
                    file_info: &file,
                });
            };

            if meteoritus.auto_terminate() {
                if let Err(_) = vault.terminate_file(id) {
                    return UploadResponder::Failure(
                        Status::InternalServerError,
                    );
                };
            }

            *file.length()
        }
    };

    UploadResponder::Success(final_offset)
}

#[derive(Debug)]
pub struct UploadRequest<'r> {
    rocket: &'r Rocket<Orbit>,
    offset: u64,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UploadRequest<'r> {
    type Error = &'static str;

    async fn from_request(
        req: &'r Request<'_>,
    ) -> request::Outcome<Self, Self::Error> {
        let tus_resumable_header = req.headers().get_one("Tus-Resumable");
        if tus_resumable_header.is_none()
            || tus_resumable_header.unwrap() != "1.0.0"
        {
            return Outcome::Error((
                Status::BadRequest,
                "Missing or invalid Tus-Resumable header",
            ));
        }

        let offset = match req.headers().get_one("Upload-Offset") {
            Some(value) => match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => {
                    return Outcome::Error((
                        Status::BadRequest,
                        "Invalid Upload-Offset header",
                    ))
                }
            },
            None => {
                return Outcome::Error((
                    Status::BadRequest,
                    "Missing Upload-Offset header",
                ))
            }
        };

        match req.content_type() {
            None => {
                return Outcome::Error((
                    Status::BadRequest,
                    "Missing Content-Type header",
                ))
            }
            Some(value)
                if value
                    != &ContentType::new(
                        "application",
                        "offset+octet-stream",
                    ) =>
            {
                return Outcome::Error((
                    Status::UnsupportedMediaType,
                    "Invalid Content-Type header",
                ))
            }
            Some(_) => (),
        };

        let upload_values = UploadRequest {
            rocket: req.rocket(),
            offset,
        };

        Outcome::Success(upload_values)
    }
}

pub enum UploadResponder {
    Success(u64),
    Failure(Status),
}

impl<'r> Responder<'r, 'static> for UploadResponder {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let meteoritus = req.rocket().state::<Meteoritus<Orbit>>().unwrap();

        let mut res = rocket::Response::build();

        res.header(meteoritus.get_protocol_resumable_version());

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
