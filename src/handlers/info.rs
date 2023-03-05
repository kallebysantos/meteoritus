use rocket::{http::Status, response::Responder, Request, Response};

use crate::Meteoritus;

#[options("/")]
pub fn info_handler() -> InfoResponder {
    InfoResponder {}
}

pub struct InfoResponder {}

impl<'r> Responder<'r, 'static> for InfoResponder {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let meteoritus = req.rocket().state::<Meteoritus>().unwrap();

        Response::build()
            .header(Meteoritus::get_protocol_resumable_version())
            .header(Meteoritus::get_protocol_version())
            .header(Meteoritus::get_protocol_extensions())
            .header(Meteoritus::get_protocol_max_size(&meteoritus))
            .status(Status::NoContent)
            .ok()
    }
}
