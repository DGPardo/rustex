// mod currencies;
pub mod db;
mod models;
mod order_matching;

pub mod prelude;

/// Macro to acquire an exclusive lock
/// ignoring any poison error
#[macro_export]
macro_rules! lock {
    ($e:expr) => {
        $e.lock().unwrap_or_else(|p| {
            log::warn!("Ignoring poisoned lock at: {}:{}", file!(), line!());
            p.into_inner()
        })
    };
}
