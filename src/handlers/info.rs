use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    response::Responder,
    Request, Response,
};

use crate::Meteoritus;

#[options("/")]
pub fn info_handler(_req: InfoRequest) -> InfoResponder {
    InfoResponder {}
}

#[derive(Debug)]
pub struct InfoRequest<'a> {
    value: &'a str,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for InfoRequest<'_> {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let info = InfoRequest {
            value: "Hello From InfoRequest Guard",
        };

        println!("{:?}", req);
        Outcome::Success(info)
    }
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
