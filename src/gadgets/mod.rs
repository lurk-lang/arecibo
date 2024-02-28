//! This module implements various gadgets necessary for Nova and applications built with Nova.
mod ecc;
pub(crate) use ecc::AllocatedPoint;

mod nonnative;
pub(crate) use nonnative::{bignat::nat_to_limbs, util::f_to_nat};

mod r1cs;
pub(crate) use r1cs::{
  conditionally_select_alloc_relaxed_r1cs, conditionally_select_vec_allocated_relaxed_r1cs_instance,
};
pub(crate) use r1cs::{AllocatedR1CSInstance, AllocatedRelaxedR1CSInstance};

pub(crate) mod utils;
