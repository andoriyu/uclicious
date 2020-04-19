//! Safe implementations of vars handler from libUCL.
#[cfg(feature = "vh_compound")]
pub mod compound;
#[cfg(feature = "vh_basic")]
pub mod env;
