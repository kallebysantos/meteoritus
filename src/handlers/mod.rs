mod creation;
mod file_info;
mod info;
// mod termination;
mod upload;

pub use creation::creation_handler;
pub use file_info::file_info_handler;
pub use info::info_handler;
use rocket::{Orbit, Rocket};
// pub use termination::termination_handler;
pub use upload::upload_handler;

use crate::fs::FileInfo;

#[derive(Debug)]
pub struct HandlerContext<'a, S> {
    pub rocket: &'a Rocket<Orbit>,
    pub file_info: &'a FileInfo<S>,
}
