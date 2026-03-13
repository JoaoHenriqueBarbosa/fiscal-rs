mod adjust;
mod manager;

#[cfg(test)]
mod tests;

pub use adjust::adjust_nfe_contingency;
pub use manager::{Contingency, contingency_for_state, try_contingency_for_state};
