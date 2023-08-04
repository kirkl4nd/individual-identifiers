//! A Rust library for generating unique, alliterative identifiers.
//! Each identifier consists of a UUID, and an alliterative, human-readable name made up of an adjective and a noun.
//! This library is perfect for creating memorable, user-friendly identifiers that also include a UUID for uniqueness and easy database indexing.

mod identifier;

pub use identifier::Identifier;