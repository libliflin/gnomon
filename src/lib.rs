pub mod analyze;
pub mod audit;
pub mod budget;
pub mod cli;
pub mod fetch;
pub mod forbidden;
pub mod report;
pub mod violation;

pub use audit::{AuditReport, audit_url};
pub use budget::{Budget, Preset};
pub use violation::{Violation, ViolationKind};

#[cfg(test)]
mod tests {
    #[test]
    fn sanity() {
        assert!(true);
    }
}
