use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    response::Responder,
    Request, Response,
};

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
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .raw_header("Tus-Version", "1.0.0")
            .raw_header("Tus-Max-Size", "5120")
            .raw_header("Tus-Extension", "creation,expiration,termination")
            .status(Status::NoContent)
            .ok()
    }
}
