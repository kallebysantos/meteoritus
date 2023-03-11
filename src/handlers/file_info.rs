use rocket::{
    http::Status,
    response::{self, Responder},
    Request, State,
};

use crate::{CometFile, Meteoritus};

#[head("/<id>")]
pub fn file_info_handler(id: &str, meteoritus: &State<Meteoritus>) -> FileInfoResponder {
    match meteoritus.vault.take(id.to_string()) {
        Ok(file) => FileInfoResponder::Success(file),
        Err(_) => FileInfoResponder::Failure(Status::NotFound),
    }
}

pub enum FileInfoResponder {
    Success(CometFile),
    Failure(Status),
}

impl<'r> Responder<'r, 'static> for FileInfoResponder {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let mut res = rocket::Response::build();

        res.header(Meteoritus::get_protocol_resumable_version());

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
