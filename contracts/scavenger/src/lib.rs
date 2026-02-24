#![no_std]

mod contract;
mod storage;
mod test;
mod events;
mod types;

pub use contract::*;
// This ensures that all types, including the new GlobalMetrics, 
// are exported and available for tests and external queries.
pub use types::*;