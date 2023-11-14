use bellpepper_core::{
  boolean::Boolean, num::AllocatedNum, ConstraintSystem, LinearCombination, SynthesisError,
};
use ff::PrimeField;

use crate::{
  gadgets::{
    r1cs::{conditionally_select_alloc_relaxed_r1cs, AllocatedRelaxedR1CSInstance},
    utils::alloc_num_equals_const,
  },
  traits::Group,
};

/// Return the element of `a` given by the indicator bit in `selector_vec`.
///
/// This function assumes `selector_vec` has been properly constrained", i.e. that exactly one entry is equal to 1.  
//
// NOTE: When `a` is greater than 5 (estimated), it will be cheaper to use a multicase gadget.
//
// We should plan to rely on a well-designed gadget offering a common interface but that adapts its implementation based
// on the size of inputs (known at synthesis time). The threshold size depends on the size of the elements of `a`. The
// larger the elements, the fewer are needed before multicase becomes cost-effective.
pub fn get_from_vec_alloc_relaxed_r1cs<G: Group, CS: ConstraintSystem<<G as Group>::Base>>(
  mut cs: CS,
  a: &[AllocatedRelaxedR1CSInstance<G>],
  selector_vec: &[Boolean],
) -> Result<AllocatedRelaxedR1CSInstance<G>, SynthesisError> {
  assert_eq!(a.len(), selector_vec.len());

  // Compare all instances in `a` to the first one
  let first = a
    .get(0)
    .ok_or_else(|| SynthesisError::IncompatibleLengthVector("empty vec length".to_string()))?;

  // Since `selector_vec` is correct, only one entry is 1.
  // If selector_vec[0] is 1, then all `conditionally_select` will return `first`.
  // Otherwise, the correct instance will be selected.
  let selected = a
    .iter()
    .zip(selector_vec.iter())
    .enumerate()
    .skip(1)
    .try_fold(first.clone(), |matched, (i, (candidate, equal_bit))| {
      let next_matched_allocated = conditionally_select_alloc_relaxed_r1cs(
        cs.namespace(|| format!("next_matched_allocated-{:?}", i)),
        candidate,
        &matched,
        equal_bit,
      )?;

      Ok::<AllocatedRelaxedR1CSInstance<G>, SynthesisError>(next_matched_allocated)
    })?;

  Ok(selected)
}

/// Compute a selector vector `s` of size `num_indices`, such that
/// `s[i] == 1` if i = `target_index` and 0 otherwise.
/// Returns a SynthesisError if `target_index` is not in the range 0..num_indices.
pub fn get_selector_vec_from_index<F: PrimeField, CS: ConstraintSystem<F>>(
  mut cs: CS,
  target_index: &AllocatedNum<F>,
  num_indices: usize,
) -> Result<Vec<Boolean>, SynthesisError> {
  assert_ne!(num_indices, 0);

  // Trivial case, return [1]
  if num_indices == 1 {
    return Ok(vec![Boolean::Constant(true)]);
  }

  let selector = (0..num_indices)
    .map(|idx| {
      // b <- idx == target_index
      Ok(Boolean::Is(alloc_num_equals_const(
        cs.namespace(|| format!("check {:?} equal bit", idx)),
        target_index,
        F::from(idx as u64),
      )?))
    })
    .collect::<Result<Vec<Boolean>, SynthesisError>>()?;

  // Enforce ∑selector[i] = 1
  let selected_sum = selector
    .iter()
    .fold(LinearCombination::default(), |lc, bit| {
      lc + &bit.lc(CS::one(), F::ONE)
    });

  cs.enforce(
    || "exactly-one-selection",
    |_| selected_sum,
    |lc| lc + CS::one(),
    |lc| lc + CS::one(),
  );

  Ok(selector)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::gadgets::utils::alloc_const;
  use bellpepper_core::test_cs::TestConstraintSystem;
  use pasta_curves::pallas::{Base, Point};

  #[test]
  fn test_get_from_vec_alloc_relaxed_r1cs_bounds() {
    let n = 3;
    for selected in 0..(2 * n) {
      let mut cs = TestConstraintSystem::<Base>::new();

      let allocated_target =
        alloc_const(&mut cs.namespace(|| "target"), Base::from(selected as u64)).unwrap();

      let selector_vec = get_selector_vec_from_index(&mut cs, &allocated_target, n).unwrap();

      let vec = (0..n)
        .map(|i| {
          AllocatedRelaxedR1CSInstance::<Point>::default(
            &mut cs.namespace(|| format!("elt-{i}")),
            4,
            64,
          )
          .unwrap()
        })
        .collect::<Vec<_>>();

      get_from_vec_alloc_relaxed_r1cs(&mut cs.namespace(|| "test-fn"), &vec, &selector_vec)
        .unwrap();

      if selected < n {
        assert!(cs.is_satisfied())
      } else {
        // If selected is out of range, the circuit must be unsatisfied.
        assert!(!cs.is_satisfied())
      }
    }
  }

  #[test]
  fn test_get_selector() {
    for n in 1..4 {
      for selected in 0..n {
        let mut cs = TestConstraintSystem::<Base>::new();

        let allocated_target =
          alloc_const(&mut cs.namespace(|| "target"), Base::from(selected as u64)).unwrap();

        let selector_vec = get_selector_vec_from_index(&mut cs, &allocated_target, n).unwrap();

        let is_valid = selector_vec
          .iter()
          .try_fold(false, |res, bit| bit.get_value().map(|bit| res | bit))
          .unwrap();

        if selected < n {
          assert!(cs.is_satisfied());
          assert!(is_valid);
        } else {
          // If selected is out of range, the circuit must be unsatisfied.
          assert!(!cs.is_satisfied());
          assert!(!is_valid);
        }
      }
    }
  }
}
