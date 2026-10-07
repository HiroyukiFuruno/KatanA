mod definitions;
mod face_identity;
mod manager;

#[cfg(test)]
mod tests;

pub(crate) use manager::{DocumentFontLease, DocumentFontLeaseManager};
