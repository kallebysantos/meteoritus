use rocket::{
    http::Status,
    response::{self, Responder},
    Orbit, Request, State,
};

use crate::{meteoritus::Meteoritus, CometFile};

#[head("/<id>")]
pub fn file_info_handler(id: &str, meteoritus: &State<Meteoritus<Orbit>>) -> FileInfoResponder {
    match meteoritus.vault().take(id.to_string()) {
        Ok(file) => FileInfoResponder::Success(file),
        Err(_) => FileInfoResponder::Failure(Status::NotFound),
    }
}

pub enum FileInfoResponder {
    Success(CometFile),
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
