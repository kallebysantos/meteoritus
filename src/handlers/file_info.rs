use std::sync::Arc;

use rocket::{
    http::Status,
    response::{self, Responder},
    Orbit, Request, State,
};

use crate::{
    fs::{Created, FileInfo},
    meteoritus::Meteoritus,
    Vault,
};

#[head("/<id>")]
pub fn file_info_handler(
    id: &str,
    vault: &State<Arc<dyn Vault>>,
) -> FileInfoResponder {
    match vault.get_file(id) {
        Ok(file) => FileInfoResponder::Success(file),
        Err(_) => FileInfoResponder::Failure(Status::NotFound),
    }
}

pub enum FileInfoResponder {
    Success(FileInfo<Created>),
    Failure(Status),
}

impl<'r> Responder<'r, 'static> for FileInfoResponder {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let meteoritus = req.rocket().state::<Meteoritus<Orbit>>().unwrap();

        let mut res = rocket::Response::build();

        res.header(meteoritus.get_protocol_resumable_version());

        match self {
            Self::Success(file) => {
                res.status(Status::NoContent);
                res.raw_header("Upload-Length", file.length().to_string());
                res.raw_header("Upload-Offset", file.offset().to_string())
            }
            Self::Failure(status) => res.status(status),
        };

        res.ok()
    }
}
