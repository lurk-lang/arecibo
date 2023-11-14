//! This module implements various low-level gadgets
use super::nonnative::bignat::{nat_to_limbs, BigNat};
use crate::traits::Group;
use bellpepper::gadgets::Assignment;
use bellpepper_core::{
  boolean::{AllocatedBit, Boolean},
  num::AllocatedNum,
  ConstraintSystem, LinearCombination, SynthesisError,
};
use ff::{Field, PrimeField, PrimeFieldBits};
use num_bigint::BigInt;

/// Gets as input the little indian representation of a number and spits out the number
pub fn le_bits_to_num<Scalar, CS>(
  mut cs: CS,
  bits: &[AllocatedBit],
) -> Result<AllocatedNum<Scalar>, SynthesisError>
where
  Scalar: PrimeField + PrimeFieldBits,
  CS: ConstraintSystem<Scalar>,
{
  // We loop over the input bits and construct the constraint
  // and the field element that corresponds to the result
  let mut lc = LinearCombination::zero();
  let mut coeff = Scalar::ONE;
  let mut fe = Some(Scalar::ZERO);
  for bit in bits.iter() {
    lc = lc + (coeff, bit.get_variable());
    fe = bit.get_value().map(|val| {
      if val {
        fe.unwrap() + coeff
      } else {
        fe.unwrap()
      }
    });
    coeff = coeff.double();
  }
  let num = AllocatedNum::alloc(cs.namespace(|| "Field element"), || {
    fe.ok_or(SynthesisError::AssignmentMissing)
  })?;
  lc = lc - num.get_variable();
  cs.enforce(|| "compute number from bits", |lc| lc, |lc| lc, |_| lc);
  Ok(num)
}

/// Allocate a variable that is set to zero
pub fn alloc_zero<F: PrimeField, CS: ConstraintSystem<F>>(mut cs: CS) -> AllocatedNum<F> {
  let zero = AllocatedNum::alloc_infallible(cs.namespace(|| "alloc"), || F::ZERO);
  cs.enforce(
    || "check zero is valid",
    |lc| lc,
    |lc| lc,
    |lc| lc + zero.get_variable(),
  );
  zero
}

/// Allocate a variable that is set to one
pub fn alloc_one<F: PrimeField, CS: ConstraintSystem<F>>(mut cs: CS) -> AllocatedNum<F> {
  let one = AllocatedNum::alloc_infallible(cs.namespace(|| "alloc"), || F::ONE);
  cs.enforce(
    || "check one is valid",
    |lc| lc + CS::one(),
    |lc| lc + CS::one(),
    |lc| lc + one.get_variable(),
  );

  one
}

/// alloc a field as a constant
/// implemented refer from <https://github.com/lurk-lab/lurk-rs/blob/4335fbb3290ed1a1176e29428f7daacb47f8033d/src/circuit/gadgets/data.rs#L387-L402>
#[allow(unused)]
pub fn alloc_const<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  val: F,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let allocated = AllocatedNum::<F>::alloc(cs.namespace(|| "allocate const"), || Ok(val))?;

  // allocated * 1 = val
  cs.enforce(
    || "enforce constant",
    |lc| lc + allocated.get_variable(),
    |lc| lc + CS::one(),
    |_| Boolean::Constant(true).lc(CS::one(), val),
  );

  Ok(allocated)
}

/// Allocate a scalar as a base. Only to be used if the scalar fits in base!
pub fn alloc_scalar_as_base<G, CS>(
  mut cs: CS,
  input: Option<G::Scalar>,
) -> Result<AllocatedNum<G::Base>, SynthesisError>
where
  G: Group,
  <G as Group>::Scalar: PrimeFieldBits,
  CS: ConstraintSystem<<G as Group>::Base>,
{
  AllocatedNum::alloc(cs.namespace(|| "allocate scalar as base"), || {
    let input_bits = input.unwrap_or(G::Scalar::ZERO).clone().to_le_bits();
    let mut mult = G::Base::ONE;
    let mut val = G::Base::ZERO;
    for bit in input_bits {
      if bit {
        val += mult;
      }
      mult = mult + mult;
    }
    Ok(val)
  })
}

/// interepret scalar as base
pub fn scalar_as_base<G: Group>(input: G::Scalar) -> G::Base {
  let input_bits = input.to_le_bits();
  let mut mult = G::Base::ONE;
  let mut val = G::Base::ZERO;
  for bit in input_bits {
    if bit {
      val += mult;
    }
    mult = mult + mult;
  }
  val
}

/// Allocate bignat a constant
pub fn alloc_bignat_constant<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  val: &BigInt,
  limb_width: usize,
  n_limbs: usize,
) -> Result<BigNat<F>, SynthesisError> {
  let limbs = nat_to_limbs(val, limb_width, n_limbs).unwrap();
  let bignat = BigNat::alloc_from_limbs(
    cs.namespace(|| "alloc bignat"),
    || Ok(limbs.clone()),
    None,
    limb_width,
    n_limbs,
  )?;
  // Now enforce that the limbs are all equal to the constants
  (0..n_limbs).for_each(|i| {
    cs.enforce(
      || format!("check limb {i}"),
      |lc| lc + &bignat.limbs[i],
      |lc| lc + CS::one(),
      |lc| lc + (limbs[i], CS::one()),
    );
  });
  Ok(bignat)
}

/// Check that two numbers are equal and return a bit
pub fn alloc_num_equals<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  b: &AllocatedNum<F>,
) -> Result<AllocatedBit, SynthesisError> {
  // Allocate and constrain `r`: result boolean bit.
  // It equals `true` if `a` equals `b`, `false` otherwise
  let r_value = match (a.get_value(), b.get_value()) {
    (Some(a), Some(b)) => Some(a == b),
    _ => None,
  };

  let r = AllocatedBit::alloc(cs.namespace(|| "r"), r_value)?;

  // Allocate t s.t. t=1 if a == b else 1/(a - b)

  let t = AllocatedNum::alloc(cs.namespace(|| "t"), || {
    let a_val = *a.get_value().get()?;
    let b_val = *b.get_value().get()?;
    Ok(if a_val == b_val {
      F::ONE
    } else {
      (a_val - b_val).invert().unwrap()
    })
  })?;

  cs.enforce(
    || "t*(a - b) = 1 - r",
    |lc| lc + t.get_variable(),
    |lc| lc + a.get_variable() - b.get_variable(),
    |lc| lc + CS::one() - r.get_variable(),
  );

  cs.enforce(
    || "r*(a - b) = 0",
    |lc| lc + r.get_variable(),
    |lc| lc + a.get_variable() - b.get_variable(),
    |lc| lc,
  );

  Ok(r)
}

/// Check that two numbers are equal and return a bit
pub fn alloc_num_equals_const<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  b: F,
) -> Result<AllocatedBit, SynthesisError> {
  // Allocate and constrain `r`: result boolean bit.
  // It equals `true` if `a` equals `b`, `false` otherwise
  let r_value = a.get_value().map(|a_val| a_val == b);

  let r = AllocatedBit::alloc(cs.namespace(|| "r"), r_value)?;

  // Allocate t s.t. t=1 if a == b else 1/(a - b)

  let t = AllocatedNum::alloc(cs.namespace(|| "t"), || {
    let a_val = *a.get_value().get()?;
    Ok(if a_val == b {
      F::ONE
    } else {
      (a_val - b).invert().unwrap()
    })
  })?;

  cs.enforce(
    || "t*(a - b) = 1 - r",
    |lc| lc + t.get_variable(),
    |lc| lc + a.get_variable() - (b, CS::one()),
    |lc| lc + CS::one() - r.get_variable(),
  );

  cs.enforce(
    || "r*(a - b) = 0",
    |lc| lc + r.get_variable(),
    |lc| lc + a.get_variable() - (b, CS::one()),
    |lc| lc,
  );

  Ok(r)
}

/// If condition return a otherwise b
pub fn conditionally_select<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  b: &AllocatedNum<F>,
  condition: &Boolean,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let c = AllocatedNum::alloc(cs.namespace(|| "conditional select result"), || {
    if *condition.get_value().get()? {
      Ok(*a.get_value().get()?)
    } else {
      Ok(*b.get_value().get()?)
    }
  })?;

  // a * condition + b*(1-condition) = c ->
  // a * condition - b*condition = c - b
  cs.enforce(
    || "conditional select constraint",
    |lc| lc + a.get_variable() - b.get_variable(),
    |_| condition.lc(CS::one(), F::ONE),
    |lc| lc + c.get_variable() - b.get_variable(),
  );

  Ok(c)
}

/// If condition return a otherwise b
pub fn conditionally_select_vec<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &[AllocatedNum<F>],
  b: &[AllocatedNum<F>],
  condition: &Boolean,
) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
  a.iter()
    .zip(b.iter())
    .enumerate()
    .map(|(i, (a, b))| {
      conditionally_select(cs.namespace(|| format!("select_{i}")), a, b, condition)
    })
    .collect::<Result<Vec<AllocatedNum<F>>, SynthesisError>>()
}

/// If condition return a otherwise b where a and b are `BigNats`
pub fn conditionally_select_bignat<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &BigNat<F>,
  b: &BigNat<F>,
  condition: &Boolean,
) -> Result<BigNat<F>, SynthesisError> {
  assert!(a.limbs.len() == b.limbs.len());
  let c = BigNat::alloc_from_nat(
    cs.namespace(|| "conditional select result"),
    || {
      if *condition.get_value().get()? {
        Ok(a.value.get()?.clone())
      } else {
        Ok(b.value.get()?.clone())
      }
    },
    a.params.limb_width,
    a.params.n_limbs,
  )?;

  // a * condition + b*(1-condition) = c ->
  // a * condition - b*condition = c - b
  for i in 0..c.limbs.len() {
    cs.enforce(
      || format!("conditional select constraint {i}"),
      |lc| lc + &a.limbs[i] - &b.limbs[i],
      |_| condition.lc(CS::one(), F::ONE),
      |lc| lc + &c.limbs[i] - &b.limbs[i],
    );
  }
  Ok(c)
}

/// Same as the above but Condition is an `AllocatedNum` that needs to be
/// 0 or 1. 1 => True, 0 => False
pub fn conditionally_select2<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  b: &AllocatedNum<F>,
  condition: &AllocatedNum<F>,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let c = AllocatedNum::alloc(cs.namespace(|| "conditional select result"), || {
    if *condition.get_value().get()? == F::ONE {
      Ok(*a.get_value().get()?)
    } else {
      Ok(*b.get_value().get()?)
    }
  })?;

  // a * condition + b*(1-condition) = c ->
  // a * condition - b*condition = c - b
  cs.enforce(
    || "conditional select constraint",
    |lc| lc + a.get_variable() - b.get_variable(),
    |lc| lc + condition.get_variable(),
    |lc| lc + c.get_variable() - b.get_variable(),
  );

  Ok(c)
}

/// If condition set to 0 otherwise a. Condition is an allocated num
pub fn select_zero_or_num2<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  condition: &AllocatedNum<F>,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let c = AllocatedNum::alloc(cs.namespace(|| "conditional select result"), || {
    if *condition.get_value().get()? == F::ONE {
      Ok(F::ZERO)
    } else {
      Ok(*a.get_value().get()?)
    }
  })?;

  // a * (1 - condition) = c
  cs.enforce(
    || "conditional select constraint",
    |lc| lc + a.get_variable(),
    |lc| lc + CS::one() - condition.get_variable(),
    |lc| lc + c.get_variable(),
  );

  Ok(c)
}

/// If condition set to a otherwise 0. Condition is an allocated num
pub fn select_num_or_zero2<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  condition: &AllocatedNum<F>,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let c = AllocatedNum::alloc(cs.namespace(|| "conditional select result"), || {
    if *condition.get_value().get()? == F::ONE {
      Ok(*a.get_value().get()?)
    } else {
      Ok(F::ZERO)
    }
  })?;

  cs.enforce(
    || "conditional select constraint",
    |lc| lc + a.get_variable(),
    |lc| lc + condition.get_variable(),
    |lc| lc + c.get_variable(),
  );

  Ok(c)
}

/// If condition set to a otherwise 0
pub fn select_num_or_zero<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  condition: &Boolean,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let c = AllocatedNum::alloc(cs.namespace(|| "conditional select result"), || {
    if *condition.get_value().get()? {
      Ok(*a.get_value().get()?)
    } else {
      Ok(F::ZERO)
    }
  })?;

  cs.enforce(
    || "conditional select constraint",
    |lc| lc + a.get_variable(),
    |_| condition.lc(CS::one(), F::ONE),
    |lc| lc + c.get_variable(),
  );

  Ok(c)
}

/// If condition set to 1 otherwise a
pub fn select_one_or_num2<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  condition: &AllocatedNum<F>,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let c = AllocatedNum::alloc(cs.namespace(|| "conditional select result"), || {
    if *condition.get_value().get()? == F::ONE {
      Ok(F::ONE)
    } else {
      Ok(*a.get_value().get()?)
    }
  })?;

  cs.enforce(
    || "conditional select constraint",
    |lc| lc + CS::one() - a.get_variable(),
    |lc| lc + condition.get_variable(),
    |lc| lc + c.get_variable() - a.get_variable(),
  );
  Ok(c)
}

/// If condition set to 1 otherwise a - b
pub fn select_one_or_diff2<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  b: &AllocatedNum<F>,
  condition: &AllocatedNum<F>,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let c = AllocatedNum::alloc(cs.namespace(|| "conditional select result"), || {
    if *condition.get_value().get()? == F::ONE {
      Ok(F::ONE)
    } else {
      Ok(*a.get_value().get()? - *b.get_value().get()?)
    }
  })?;

  cs.enforce(
    || "conditional select constraint",
    |lc| lc + CS::one() - a.get_variable() + b.get_variable(),
    |lc| lc + condition.get_variable(),
    |lc| lc + c.get_variable() - a.get_variable() + b.get_variable(),
  );
  Ok(c)
}

/// If condition set to a otherwise 1 for boolean conditions
pub fn select_num_or_one<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  a: &AllocatedNum<F>,
  condition: &Boolean,
) -> Result<AllocatedNum<F>, SynthesisError> {
  let c = AllocatedNum::alloc(cs.namespace(|| "conditional select result"), || {
    if *condition.get_value().get()? {
      Ok(*a.get_value().get()?)
    } else {
      Ok(F::ONE)
    }
  })?;

  cs.enforce(
    || "conditional select constraint",
    |lc| lc + a.get_variable() - CS::one(),
    |_| condition.lc(CS::one(), F::ONE),
    |lc| lc + c.get_variable() - CS::one(),
  );

  Ok(c)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::r1cs::util::FWrap;
  use bellpepper_core::test_cs::TestConstraintSystem;
  use pasta_curves::pallas::Scalar as Fr;
  use proptest::prelude::*;

  proptest! {
    #[test]
    fn test_enforce_alloc_num_equal_const((a, b) in any::<(FWrap<Fr>, FWrap<Fr>)>()) {
      prop_assume!(a != b);

        let test_a_b = |a, b| {
            let mut cs = TestConstraintSystem::<Fr>::new();
            let a_num = AllocatedNum::alloc_infallible(cs.namespace(|| "a_num"), || a);
            let r = alloc_num_equals_const(&mut cs, &a_num, b);
            assert_eq!(r.unwrap().get_value().unwrap(), a==b);
        };
        // negative testing
        test_a_b(a.0, b.0);
        // positive testing
        test_a_b(a.0, a.0);
    }
  }
}
