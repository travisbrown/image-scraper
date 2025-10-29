#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::missing_errors_doc)]
#![forbid(unsafe_code)]
use chrono::{DateTime, Utc};

pub mod db;
pub mod timestamp;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Entry {
    pub timestamp: DateTime<Utc>,
    pub digest: md5::Digest,
    pub image_type: imghdr::Type,
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .reverse()
            .then_with(|| self.digest.0.cmp(&other.digest.0))
            .then_with(|| self.image_type.cmp(&other.image_type))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
