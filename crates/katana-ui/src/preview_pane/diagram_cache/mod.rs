mod content;
mod metrics;
mod store;
#[cfg(test)]
mod store_tests;
mod svg_store;

pub(crate) use store::DiagramRenderCacheCoordinator;
