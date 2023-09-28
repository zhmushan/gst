use git2::{Direction, Oid, Remote};
use log::debug;

use crate::error::AnyError;

pub fn get_head(url: &str) -> Result<Oid, AnyError> {
    let mut remote = Remote::create_detached(url)?;
    remote.connect(Direction::Fetch)?;
    let head = remote
        .list()?
        .iter()
        .find(|&it| it.name() == "HEAD")
        .unwrap();
    let oid = head.oid();
    remote.disconnect()?;

    debug!("head: {}", oid);

    Ok(oid)
}
