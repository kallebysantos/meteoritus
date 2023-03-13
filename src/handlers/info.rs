use rocket::{http::Status, response::Responder, Orbit, Request, Response};

use crate::meteoritus::Meteoritus;

#[options("/")]
pub fn info_handler() -> InfoResponder {
    InfoResponder {}
}

pub struct InfoResponder {}

impl<'r> Responder<'r, 'static> for InfoResponder {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let meteoritus = req.rocket().state::<Meteoritus<Orbit>>().unwrap();

        Response::build()
            .header(meteoritus.get_protocol_resumable_version())
            .header(meteoritus.get_protocol_version())
            .header(meteoritus.get_protocol_extensions())
            .header(meteoritus.get_protocol_max_size())
            .status(Status::NoContent)
            .ok()
    }
}
