mod host_backend;

mod sandbox_backend;
pub use sandbox_backend::SandboxBackend;

use crate::backend::PackageTransaction;

pub trait TransactionBackend {
    fn new() -> Self
    where
        Self: Sized;

    fn add_package_transaction(&self, transaction: std::sync::Arc<PackageTransaction>);

    fn cancel_package_transaction(&self, transaction: std::sync::Arc<PackageTransaction>);
}
