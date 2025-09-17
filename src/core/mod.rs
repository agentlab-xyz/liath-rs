mod fjall_wrapper;
mod namespace;

pub use fjall_wrapper::FjallWrapper;
pub use namespace::{Namespace, NamespaceManager};
#[cfg(not(feature = "vector"))]
pub use namespace::{MetricKind, ScalarKind};
