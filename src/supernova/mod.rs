//! This library implements `SuperNova`, a Non-Uniform IVC based on Nova.

use std::marker::PhantomData;
use std::ops::Index;
use std::{cmp::max, io};

use crate::{
  bellpepper::shape_cs::ShapeCS,
  constants::{BN_LIMB_WIDTH, BN_N_LIMBS, NUM_HASH_BITS},
  digest::{DigestComputer, Digestible, SimpleDigestible},
  errors::NovaError,
  r1cs::{
    commitment_key_size, R1CSInstance, R1CSShape, R1CSWitness, RelaxedR1CSInstance,
    RelaxedR1CSWitness,
  },
  scalar_as_base,
  traits::{
    circuit_supernova::{EnforcingStepCircuit, StepCircuit, TrivialSecondaryCircuit},
    commitment::{CommitmentEngineTrait, CommitmentTrait, Len},
    AbsorbInROTrait, Group, ROConstants, ROConstantsCircuit, ROTrait,
  },
  Commitment, CommitmentKey,
};

use abomonation::Abomonation;
use abomonation_derive::Abomonation;
use ff::{Field, PrimeField};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::bellpepper::{
  r1cs::{NovaShape, NovaWitness},
  solver::SatisfyingAssignment,
};
use bellpepper_core::ConstraintSystem;

use crate::nifs::NIFS;

mod circuit; // declare the module first
use circuit::{
  SuperNovaAugmentedCircuit, SuperNovaAugmentedCircuitInputs, SuperNovaAugmentedCircuitParams,
};

use self::error::SuperNovaError;

pub mod error;
pub(crate) mod utils;

#[cfg(test)]
mod test;

/// A type that holds public parameters of Nova
#[derive(Clone, PartialEq, Serialize, Deserialize, Abomonation)]
#[serde(bound = "")]
#[abomonation_bounds(
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  <G1::Scalar as PrimeField>::Repr: Abomonation,
  <G2::Scalar as PrimeField>::Repr: Abomonation,
)]

pub struct PublicParams<G1, G2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
{
  F_arity_primary: usize,
  F_arity_secondary: usize,
  ro_consts_primary: ROConstants<G1>,
  ro_consts_circuit_primary: ROConstantsCircuit<G2>,
  ck_primary: Option<CommitmentKey<G1>>, // This is shared between all public params of the `RunningClaims`
  r1cs_shape_primary: R1CSShape<G1>,
  ro_consts_secondary: ROConstants<G2>,
  ro_consts_circuit_secondary: ROConstantsCircuit<G1>,
  ck_secondary: Option<CommitmentKey<G2>>, // This is shared between all public params of the `RunningClaims`
  r1cs_shape_secondary: R1CSShape<G2>,
  augmented_circuit_params_primary: SuperNovaAugmentedCircuitParams,
  augmented_circuit_params_secondary: SuperNovaAugmentedCircuitParams,
}

impl<G1, G2> SimpleDigestible for PublicParams<G1, G2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
{
}

impl<G1, G2> PublicParams<G1, G2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
{
  /// Create a new `PublicParams`
  pub fn setup_without_commitkey<
    C1: EnforcingStepCircuit<G1::Scalar>,
    C2: EnforcingStepCircuit<G2::Scalar>,
  >(
    c_primary: &C1,
    c_secondary: &C2,
    num_augmented_circuits: usize,
  ) -> Self where {
    let augmented_circuit_params_primary =
      SuperNovaAugmentedCircuitParams::new(BN_LIMB_WIDTH, BN_N_LIMBS, true);
    let augmented_circuit_params_secondary =
      SuperNovaAugmentedCircuitParams::new(BN_LIMB_WIDTH, BN_N_LIMBS, false);

    let ro_consts_primary: ROConstants<G1> = ROConstants::<G1>::default();
    let ro_consts_secondary: ROConstants<G2> = ROConstants::<G2>::default();

    let F_arity_primary = c_primary.arity();
    let F_arity_secondary = c_secondary.arity();

    // ro_consts_circuit_primary are parameterized by G2 because the type alias uses G2::Base = G1::Scalar
    let ro_consts_circuit_primary: ROConstantsCircuit<G2> = ROConstantsCircuit::<G2>::default();
    let ro_consts_circuit_secondary: ROConstantsCircuit<G1> = ROConstantsCircuit::<G1>::default();

    // Initialize ck for the primary
    let circuit_primary: SuperNovaAugmentedCircuit<'_, G2, C1> = SuperNovaAugmentedCircuit::new(
      &augmented_circuit_params_primary,
      None,
      c_primary,
      ro_consts_circuit_primary.clone(),
      num_augmented_circuits,
    );
    let mut cs: ShapeCS<G1> = ShapeCS::new();
    let _ = circuit_primary.synthesize(&mut cs);
    // We use the largest commitment_key for all instances
    let r1cs_shape_primary = cs.r1cs_shape();

    // Initialize ck for the secondary
    let circuit_secondary: SuperNovaAugmentedCircuit<'_, G1, C2> = SuperNovaAugmentedCircuit::new(
      &augmented_circuit_params_secondary,
      None,
      c_secondary,
      ro_consts_circuit_secondary.clone(),
      num_augmented_circuits,
    );
    let mut cs: ShapeCS<G2> = ShapeCS::new();
    let _ = circuit_secondary.synthesize(&mut cs);
    let r1cs_shape_secondary = cs.r1cs_shape();

    Self {
      F_arity_primary,
      F_arity_secondary,
      ro_consts_primary,
      ro_consts_circuit_primary,
      ck_primary: None,
      r1cs_shape_primary,
      ro_consts_secondary,
      ro_consts_circuit_secondary,
      ck_secondary: None,
      r1cs_shape_secondary,
      augmented_circuit_params_primary,
      augmented_circuit_params_secondary,
    }
  }

  #[allow(dead_code)]
  /// Returns the number of constraints in the primary and secondary circuits
  pub fn num_constraints(&self) -> (usize, usize) {
    (
      self.r1cs_shape_primary.num_cons,
      self.r1cs_shape_secondary.num_cons,
    )
  }

  #[allow(dead_code)]
  /// Returns the number of variables in the primary and secondary circuits
  pub fn num_variables(&self) -> (usize, usize) {
    (
      self.r1cs_shape_primary.num_vars,
      self.r1cs_shape_secondary.num_vars,
    )
  }
}

/// A vector of [PublicParams] corresponding to a set of [RunningClaims]
pub struct RunningClaimParams<G1, G2, C1>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: EnforcingStepCircuit<G1::Scalar>,
{
  /// The internal public params
  pub pp_vec: Vec<PublicParams<G1, G2>>,
  /// Digest constructed from these `RunningClaim`s' parameters
  // Once we serialize this struct, it's important to add this
  // annotaiton to the digest field for it to be treated properly: #[serde(skip, default = "OnceCell::new")]
  digest: OnceCell<G1::Scalar>,
  stage: Stage,
  _p: PhantomData<C1>,
}
enum Stage {
  Incomplete,
  Complete,
}

impl<G1, G2, C1> Index<usize> for RunningClaimParams<G1, G2, C1>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: EnforcingStepCircuit<G1::Scalar>,
{
  type Output = PublicParams<G1, G2>;

  fn index(&self, index: usize) -> &Self::Output {
    &self.pp_vec[index]
  }
}

impl<G1, G2, C1> Digestible for RunningClaimParams<G1, G2, C1>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: EnforcingStepCircuit<G1::Scalar>,
{
  fn write_bytes<W: Sized + io::Write>(&self, byte_sink: &mut W) -> Result<(), io::Error> {
    assert!(!self.pp_vec.is_empty());
    for pp in &self.pp_vec {
      pp.write_bytes(byte_sink)?;
    }
    Ok(())
  }
}

impl<G1, G2, C1> RunningClaimParams<G1, G2, C1>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: EnforcingStepCircuit<G1::Scalar>,
{
  /// new running claim params
  pub fn new<NC: NonUniformCircuit<G1, G2, C1>>(
    non_unifrom_circuit: &NC,
    // optfn1: Option<CommitmentKeyHint<G1>>,
    // optfn2: Option<CommitmentKeyHint<G2>>,
  ) -> Self {
    // uhhh if this aint zero this doesn't work
    let _initial_pc = non_unifrom_circuit.initial_program_counter();
    let num_circuits = non_unifrom_circuit.num_circuits();

    let c_primarys = (0..num_circuits)
      .map(|i| non_unifrom_circuit.primary_circuit(i))
      .collect::<Vec<_>>();
    let c_secondary = non_unifrom_circuit.secondary_circuit();

    let pp_vec = c_primarys
      .iter()
      .map(|c_primary| PublicParams::setup_without_commitkey(c_primary, &c_secondary, num_circuits))
      .collect::<Vec<_>>();

    let mut running_claim_params = RunningClaimParams {
      pp_vec,
      digest: OnceCell::new(),
      stage: Stage::Incomplete,
      _p: PhantomData,
    };

    running_claim_params.complete();
    running_claim_params
  }

  /// Create a [RunningClaimParams] from a vector of raw [PublicParams].
  /// We check the state of the commitment keys of the given params and
  /// realign them if necessary. If a digest is given, we assume that it
  /// will correctly match the public params after realigning them, and
  /// we do not check for its validity.
  pub fn from_pp_vec(pp_vec: Vec<PublicParams<G1, G2>>, digest: OnceCell<G1::Scalar>) -> Self {
    let num_pp = pp_vec.len();
    let primary_len = &pp_vec[0].ck_primary.as_ref().map_or(0, Len::length);
    let secondary_len = &pp_vec[0].ck_secondary.as_ref().map_or(0, Len::length);
    let num_complete = pp_vec
      .iter()
      .filter(|pp| {
        pp.ck_primary.is_some()
          && pp.ck_secondary.is_some()
          && &pp.ck_primary.as_ref().unwrap().length() == primary_len
          && &pp.ck_secondary.as_ref().unwrap().length() == secondary_len
      })
      .count();

    let stage = if num_complete == num_pp {
      Stage::Complete
    } else {
      Stage::Incomplete
    };

    let mut running_claim_params = RunningClaimParams {
      pp_vec,
      digest,
      stage,
      _p: PhantomData,
    };

    running_claim_params.complete();
    running_claim_params
  }

  /// Compute primary and secondary commitment keys sized to handle the largest of the circuits in the provided
  /// `PublicParams`.
  fn compute_commitment_keys(&self) -> (CommitmentKey<G1>, CommitmentKey<G2>) {
    macro_rules! max_size {
      ($shape_getter:ident, $ck_getter:ident) => {
        self
          .pp_vec
          .iter()
          .map(|pp| {
            let ck_len = if let Some(ck) = &pp.$ck_getter {
              ck.length()
            } else {
              0
            };
            max(ck_len, commitment_key_size(&pp.$shape_getter, None))
          })
          .max()
          .unwrap()
      };
    }

    let size_primary = max_size!(r1cs_shape_primary, ck_primary);
    let size_secondary = max_size!(r1cs_shape_secondary, ck_secondary);

    let ck_primary = G1::CE::setup(b"ck", size_primary);
    let ck_secondary = G2::CE::setup(b"ck", size_secondary);

    (ck_primary, ck_secondary)
  }

  /// Completes the set of public params
  pub fn complete(&mut self) {
    if matches!(self.stage, Stage::Complete) {
      return;
    }

    self.set_commitment_keys();
    self.digest();
  }

  /// set primary/secondary commitment key for all the params
  fn set_commitment_keys(&mut self) {
    if matches!(self.stage, Stage::Complete) {
      return;
    }
    let (ck_primary, ck_secondary) = self.compute_commitment_keys();
    for pp in self.pp_vec.iter_mut() {
      pp.ck_primary = Some(ck_primary.clone()); // TODO: the keys should be shared across all the params
      pp.ck_secondary = Some(ck_secondary.clone()); // TODO: the keys should be shared across all the params
    }
    self.stage = Stage::Complete;
  }

  /// Return the [RunningClaimParams]' digest.
  pub fn digest(&self) -> G1::Scalar {
    self
      .digest
      .get_or_try_init(|| {
        let dc = DigestComputer::new(self);
        dc.digest()
      })
      .cloned()
      .expect("Failure in retrieving digest")
  }
}

/// SuperNova takes Ui a list of running instances.
/// One instance of Ui is a struct called RunningClaim.
#[derive(Debug, Clone)]
pub struct RunningClaim<G1, G2, C1, C2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: EnforcingStepCircuit<G1::Scalar>,
  C2: EnforcingStepCircuit<G2::Scalar>,
{
  augmented_circuit_index: usize,
  c_secondary: C2,
  _phantom: PhantomData<(C1, G1, G2)>,
  // params: PublicParams<G1, G2>, I think this is the only way
}

/// Indexed list of `RunningClaim`s representing an NIVC computation in process.
pub struct RunningClaims<G1, G2, C1, C2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: StepCircuit<G1::Scalar>,
  C2: StepCircuit<G2::Scalar>,
{
  /// Indexed `RunningClaim`s
  claims: Vec<RunningClaim<G1, G2, C1, C2>>,
}

impl<G1, G2, C1, C2> Index<usize> for RunningClaims<G1, G2, C1, C2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: StepCircuit<G1::Scalar>,
  C2: StepCircuit<G2::Scalar>,
{
  type Output = RunningClaim<G1, G2, C1, C2>;

  fn index(&self, index: usize) -> &Self::Output {
    &self.claims[index]
  }
}

impl<G1, G2, C1, C2> RunningClaims<G1, G2, C1, C2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: StepCircuit<G1::Scalar>,
  C2: StepCircuit<G2::Scalar>,
{
  fn new(running_claims: Vec<RunningClaim<G1, G2, C1, C2>>) -> Self {
    RunningClaims {
      claims: running_claims,
    }
  }
}

impl<G1, G2, C1, C2> RunningClaim<G1, G2, C1, C2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: EnforcingStepCircuit<G1::Scalar>,
  C2: EnforcingStepCircuit<G2::Scalar>,
{
  /// create a running claim
  pub fn new(
    _pp: &PublicParams<G1, G2>,
    augmented_circuit_index: usize,
    _circuit_primary: C1,
    circuit_secondary: C2,
    _num_augmented_circuits: usize,
  ) -> Self {
    // The `PublicParams` reflect the primary circuit, so there is no need to hold an independent copy, since that copy
    // would lack step-specific non-deterministic hints.
    Self {
      augmented_circuit_index,
      c_secondary: circuit_secondary,
      _phantom: PhantomData,
    }
  }

  /// Get this `RunningClaim`'s augmented circuit index.
  pub fn get_circuit_index(&self) -> usize {
    self.augmented_circuit_index
  }
}

/// A SNARK that proves the correct execution of an non-uniform incremental computation
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct RecursiveSNARK<G1, G2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
{
  r_W_primary: Vec<Option<RelaxedR1CSWitness<G1>>>,
  r_U_primary: Vec<Option<RelaxedR1CSInstance<G1>>>,
  r_W_secondary: Vec<Option<RelaxedR1CSWitness<G2>>>, // usually r_W_secondary.len() == 1
  r_U_secondary: Vec<Option<RelaxedR1CSInstance<G2>>>, // usually r_U_secondary.len() == 1
  l_w_secondary: R1CSWitness<G2>,
  l_u_secondary: R1CSInstance<G2>,
  pp_digest: G1::Scalar,
  i: usize,
  zi_primary: Vec<G1::Scalar>,
  zi_secondary: Vec<G2::Scalar>,
  program_counter: G1::Scalar,
  augmented_circuit_index: usize,
  num_augmented_circuits: usize,
}

impl<G1, G2> RecursiveSNARK<G1, G2>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
{
  /// iterate base step to get new instance of recursive SNARK
  pub fn iter_base_step<
    C1: EnforcingStepCircuit<G1::Scalar>,
    C2: EnforcingStepCircuit<G2::Scalar>,
  >(
    pp: &PublicParams<G1, G2>,
    claim: &RunningClaim<G1, G2, C1, C2>,
    c_primary: &C1,
    pp_digest: G1::Scalar,
    initial_program_counter: Option<G1::Scalar>,
    first_augmented_circuit_index: usize,
    num_augmented_circuits: usize,
    z0_primary: &[G1::Scalar],
    z0_secondary: &[G2::Scalar],
  ) -> Result<Self, SuperNovaError> {
    let c_secondary = &claim.c_secondary;
    // commitment key for primary & secondary circuit
    let ck_primary = pp.ck_primary.as_ref().ok_or(SuperNovaError::MissingCK)?;
    let ck_secondary = pp.ck_secondary.as_ref().ok_or(SuperNovaError::MissingCK)?;

    if z0_primary.len() != pp.F_arity_primary || z0_secondary.len() != pp.F_arity_secondary {
      return Err(NovaError::InvalidStepOutputLength.into());
    }

    // base case for the primary
    let mut cs_primary: SatisfyingAssignment<G1> = SatisfyingAssignment::new();
    let inputs_primary: SuperNovaAugmentedCircuitInputs<'_, G2> =
      SuperNovaAugmentedCircuitInputs::new(
        scalar_as_base::<G1>(pp_digest),
        G1::Scalar::ZERO,
        z0_primary,
        None,
        None,
        None,
        None,
        initial_program_counter,
        G1::Scalar::ZERO, // set augmented circuit index selector to 0 in base case
      );

    let circuit_primary: SuperNovaAugmentedCircuit<'_, G2, C1> = SuperNovaAugmentedCircuit::new(
      &pp.augmented_circuit_params_primary,
      Some(inputs_primary),
      c_primary,
      pp.ro_consts_circuit_primary.clone(),
      num_augmented_circuits,
    );

    let (zi_primary_pc_next, zi_primary) =
      circuit_primary.synthesize(&mut cs_primary).map_err(|err| {
        debug!("err {:?}", err);
        NovaError::SynthesisError
      })?;
    if zi_primary.len() != pp.F_arity_primary {
      return Err(NovaError::InvalidStepOutputLength.into());
    }
    let (u_primary, w_primary) = cs_primary
      .r1cs_instance_and_witness(&pp.r1cs_shape_primary, ck_primary)
      .map_err(|err| {
        debug!("err {:?}", err);
        NovaError::SynthesisError
      })?;

    // base case for the secondary
    let mut cs_secondary: SatisfyingAssignment<G2> = SatisfyingAssignment::new();
    let inputs_secondary: SuperNovaAugmentedCircuitInputs<'_, G1> =
      SuperNovaAugmentedCircuitInputs::new(
        pp_digest,
        G2::Scalar::ZERO,
        z0_secondary,
        None,
        None,
        Some(&u_primary),
        None,
        None,
        G2::Scalar::from(claim.get_circuit_index() as u64),
      );
    let circuit_secondary: SuperNovaAugmentedCircuit<'_, G1, C2> = SuperNovaAugmentedCircuit::new(
      &pp.augmented_circuit_params_secondary,
      Some(inputs_secondary),
      c_secondary,
      pp.ro_consts_circuit_secondary.clone(),
      num_augmented_circuits,
    );
    let (_, zi_secondary) = circuit_secondary
      .synthesize(&mut cs_secondary)
      .map_err(|_| NovaError::SynthesisError)?;
    if zi_secondary.len() != pp.F_arity_secondary {
      return Err(NovaError::InvalidStepOutputLength.into());
    }
    let (u_secondary, w_secondary) = cs_secondary
      .r1cs_instance_and_witness(&pp.r1cs_shape_secondary, ck_secondary)
      .map_err(|_| NovaError::UnSat)?;

    // IVC proof for the primary circuit
    let l_w_primary = w_primary;
    let l_u_primary = u_primary;
    let r_W_primary = RelaxedR1CSWitness::from_r1cs_witness(&pp.r1cs_shape_primary, &l_w_primary);

    let r_U_primary =
      RelaxedR1CSInstance::from_r1cs_instance(ck_primary, &pp.r1cs_shape_primary, &l_u_primary);

    // IVC proof of the secondary circuit
    let l_w_secondary = w_secondary;
    let l_u_secondary = u_secondary;
    let r_W_secondary = vec![Some(RelaxedR1CSWitness::<G2>::default(
      &pp.r1cs_shape_secondary,
    ))];
    let r_U_secondary = vec![Some(RelaxedR1CSInstance::default(
      ck_secondary,
      &pp.r1cs_shape_secondary,
    ))];

    // Outputs of the two circuits and next program counter thus far.
    let zi_primary = zi_primary
      .iter()
      .map(|v| v.get_value().ok_or(NovaError::SynthesisError.into()))
      .collect::<Result<Vec<<G1 as Group>::Scalar>, SuperNovaError>>()?;
    let zi_primary_pc_next = zi_primary_pc_next
      .expect("zi_primary_pc_next missing")
      .get_value()
      .ok_or::<SuperNovaError>(NovaError::SynthesisError.into())?;
    let zi_secondary = zi_secondary
      .iter()
      .map(|v| v.get_value().ok_or(NovaError::SynthesisError.into()))
      .collect::<Result<Vec<<G2 as Group>::Scalar>, SuperNovaError>>()?;

    // handle the base case by initialize U_next in next round
    let r_W_primary_initial_list = (0..num_augmented_circuits)
      .map(|i| (i == first_augmented_circuit_index).then(|| r_W_primary.clone()))
      .collect::<Vec<Option<RelaxedR1CSWitness<G1>>>>();

    let r_U_primary_initial_list = (0..num_augmented_circuits)
      .map(|i| (i == first_augmented_circuit_index).then(|| r_U_primary.clone()))
      .collect::<Vec<Option<RelaxedR1CSInstance<G1>>>>();

    Ok(Self {
      r_W_primary: r_W_primary_initial_list,
      r_U_primary: r_U_primary_initial_list,
      r_W_secondary,
      r_U_secondary,
      l_w_secondary,
      l_u_secondary,
      pp_digest,
      i: 0_usize, // after base case, next iteration start from 1
      zi_primary,
      zi_secondary,
      program_counter: zi_primary_pc_next,
      augmented_circuit_index: first_augmented_circuit_index,
      num_augmented_circuits,
    })
  }
  /// executing a step of the incremental computation
  #[allow(clippy::too_many_arguments)]
  #[tracing::instrument(skip_all, name = "supernova::prove_step")]
  pub fn prove_step<C1: EnforcingStepCircuit<G1::Scalar>, C2: EnforcingStepCircuit<G2::Scalar>>(
    &mut self,
    pp: &PublicParams<G1, G2>,
    claim: &RunningClaim<G1, G2, C1, C2>,
    c_primary: &C1,
    z0_primary: &[G1::Scalar],
    z0_secondary: &[G2::Scalar],
  ) -> Result<(), SuperNovaError> {
    // First step was already done in the constructor
    if self.i == 0 {
      self.i = 1;
      return Ok(());
    }

    if self.r_U_secondary.len() != 1 || self.r_W_secondary.len() != 1 {
      return Err(NovaError::ProofVerifyError.into());
    }

    // let pp = &claim.params;
    let c_secondary = &claim.c_secondary;
    // commitment key for primary & secondary circuit
    let ck_primary = pp.ck_primary.as_ref().ok_or(SuperNovaError::MissingCK)?;
    let ck_secondary = pp.ck_secondary.as_ref().ok_or(SuperNovaError::MissingCK)?;

    if z0_primary.len() != pp.F_arity_primary || z0_secondary.len() != pp.F_arity_secondary {
      return Err(NovaError::InvalidInitialInputLength.into());
    }

    // fold the secondary circuit's instance
    let (nifs_secondary, (r_U_secondary_folded, r_W_secondary_folded)) = NIFS::prove(
      ck_secondary,
      &pp.ro_consts_secondary,
      &scalar_as_base::<G1>(self.pp_digest),
      &pp.r1cs_shape_secondary,
      self.r_U_secondary[0].as_ref().unwrap(),
      self.r_W_secondary[0].as_ref().unwrap(),
      &self.l_u_secondary,
      &self.l_w_secondary,
    )
    .map_err(SuperNovaError::NovaError)?;

    // clone and updated running instance on respective circuit_index
    let r_U_secondary_next = r_U_secondary_folded;
    let r_W_secondary_next = r_W_secondary_folded;

    let mut cs_primary: SatisfyingAssignment<G1> = SatisfyingAssignment::new();
    let T =
      Commitment::<G2>::decompress(&nifs_secondary.comm_T).map_err(SuperNovaError::NovaError)?;
    let inputs_primary: SuperNovaAugmentedCircuitInputs<'_, G2> =
      SuperNovaAugmentedCircuitInputs::new(
        scalar_as_base::<G1>(self.pp_digest),
        G1::Scalar::from(self.i as u64),
        z0_primary,
        Some(&self.zi_primary),
        Some(&self.r_U_secondary),
        Some(&self.l_u_secondary),
        Some(&T),
        Some(self.program_counter),
        G1::Scalar::ZERO,
      );

    let circuit_primary: SuperNovaAugmentedCircuit<'_, G2, C1> = SuperNovaAugmentedCircuit::new(
      &pp.augmented_circuit_params_primary,
      Some(inputs_primary),
      c_primary,
      pp.ro_consts_circuit_primary.clone(),
      self.num_augmented_circuits,
    );

    let (zi_primary_pc_next, zi_primary) = circuit_primary
      .synthesize(&mut cs_primary)
      .map_err(|_| SuperNovaError::NovaError(NovaError::SynthesisError))?;
    if zi_primary.len() != pp.F_arity_primary {
      return Err(SuperNovaError::NovaError(
        NovaError::InvalidInitialInputLength,
      ));
    }

    let (l_u_primary, l_w_primary) = cs_primary
      .r1cs_instance_and_witness(&pp.r1cs_shape_primary, ck_primary)
      .map_err(|_| SuperNovaError::NovaError(NovaError::UnSat))?;

    // Split into `if let`/`else` statement
    // to avoid `returns a value referencing data owned by closure` error on `&RelaxedR1CSInstance::default` and `RelaxedR1CSWitness::default`
    let (nifs_primary, (r_U_primary_folded, r_W_primary_folded)) = match (
      self.r_U_primary.get(claim.get_circuit_index()),
      self.r_W_primary.get(claim.get_circuit_index()),
    ) {
      (Some(Some(r_U_primary)), Some(Some(r_W_primary))) => NIFS::prove(
        ck_primary,
        &pp.ro_consts_primary,
        &self.pp_digest,
        &pp.r1cs_shape_primary,
        r_U_primary,
        r_W_primary,
        &l_u_primary,
        &l_w_primary,
      )
      .map_err(SuperNovaError::NovaError)?,
      _ => NIFS::prove(
        ck_primary,
        &pp.ro_consts_primary,
        &self.pp_digest,
        &pp.r1cs_shape_primary,
        &RelaxedR1CSInstance::default(ck_primary, &pp.r1cs_shape_primary),
        &RelaxedR1CSWitness::default(&pp.r1cs_shape_primary),
        &l_u_primary,
        &l_w_primary,
      )
      .map_err(SuperNovaError::NovaError)?,
    };

    let mut cs_secondary: SatisfyingAssignment<G2> = SatisfyingAssignment::new();
    let binding =
      Commitment::<G1>::decompress(&nifs_primary.comm_T).map_err(SuperNovaError::NovaError)?;
    let inputs_secondary: SuperNovaAugmentedCircuitInputs<'_, G1> =
      SuperNovaAugmentedCircuitInputs::new(
        self.pp_digest,
        G2::Scalar::from(self.i as u64),
        z0_secondary,
        Some(&self.zi_secondary),
        Some(&self.r_U_primary),
        Some(&l_u_primary),
        Some(&binding),
        None,
        G2::Scalar::from(claim.get_circuit_index() as u64),
      );

    let circuit_secondary: SuperNovaAugmentedCircuit<'_, G1, C2> = SuperNovaAugmentedCircuit::new(
      &pp.augmented_circuit_params_secondary,
      Some(inputs_secondary),
      c_secondary,
      pp.ro_consts_circuit_secondary.clone(),
      self.num_augmented_circuits,
    );
    let (_, zi_secondary) = circuit_secondary
      .synthesize(&mut cs_secondary)
      .map_err(|_| SuperNovaError::NovaError(NovaError::SynthesisError))?;
    if zi_secondary.len() != pp.F_arity_secondary {
      return Err(SuperNovaError::NovaError(
        NovaError::InvalidInitialInputLength,
      ));
    }

    let (l_u_secondary_next, l_w_secondary_next) = cs_secondary
      .r1cs_instance_and_witness(&pp.r1cs_shape_secondary, ck_secondary)
      .map_err(|_| SuperNovaError::NovaError(NovaError::UnSat))?;

    // update the running instances and witnesses
    let zi_primary = zi_primary
      .iter()
      .map(|v| {
        v.get_value()
          .ok_or(SuperNovaError::NovaError(NovaError::SynthesisError))
      })
      .collect::<Result<Vec<<G1 as Group>::Scalar>, SuperNovaError>>()?;
    let zi_primary_pc_next = zi_primary_pc_next
      .expect("zi_primary_pc_next missing")
      .get_value()
      .ok_or(SuperNovaError::NovaError(NovaError::SynthesisError))?;
    let zi_secondary = zi_secondary
      .iter()
      .map(|v| {
        v.get_value()
          .ok_or(SuperNovaError::NovaError(NovaError::SynthesisError))
      })
      .collect::<Result<Vec<<G2 as Group>::Scalar>, SuperNovaError>>()?;

    if zi_primary.len() != pp.F_arity_primary || zi_secondary.len() != pp.F_arity_secondary {
      return Err(SuperNovaError::NovaError(
        NovaError::InvalidStepOutputLength,
      ));
    }

    // clone and updated running instance on respective circuit_index
    self.r_U_primary[claim.get_circuit_index()] = Some(r_U_primary_folded);
    self.r_W_primary[claim.get_circuit_index()] = Some(r_W_primary_folded);
    self.r_W_secondary = vec![Some(r_W_secondary_next)];
    self.r_U_secondary = vec![Some(r_U_secondary_next)];
    self.l_w_secondary = l_w_secondary_next;
    self.l_u_secondary = l_u_secondary_next;
    self.i += 1;
    self.zi_primary = zi_primary;
    self.zi_secondary = zi_secondary;
    self.program_counter = zi_primary_pc_next;
    self.augmented_circuit_index = claim.get_circuit_index();
    Ok(())
  }

  /// verify recursive snark
  pub fn verify<C1: EnforcingStepCircuit<G1::Scalar>, C2: EnforcingStepCircuit<G2::Scalar>>(
    &self,
    pp: &PublicParams<G1, G2>,
    claim: &RunningClaim<G1, G2, C1, C2>,
    z0_primary: &[G1::Scalar],
    z0_secondary: &[G2::Scalar],
  ) -> Result<(), SuperNovaError> {
    // number of steps cannot be zero
    if self.i == 0 {
      debug!("must verify on valid RecursiveSNARK where i > 0");
      return Err(SuperNovaError::NovaError(NovaError::ProofVerifyError));
    }

    // check the (relaxed) R1CS instances public outputs.
    if self.l_u_secondary.X.len() != 2 {
      return Err(SuperNovaError::NovaError(NovaError::ProofVerifyError));
    }

    if self.r_U_secondary.len() != 1 || self.r_W_secondary.len() != 1 {
      return Err(SuperNovaError::NovaError(NovaError::ProofVerifyError));
    }

    // let pp = &claim.params;
    let ck_primary = pp.ck_primary.as_ref().ok_or(SuperNovaError::MissingCK)?;

    self.r_U_primary[claim.get_circuit_index()]
      .as_ref()
      .map_or(Ok(()), |U| {
        if U.X.len() != 2 {
          debug!("r_U_primary got instance length {:?} != {:?}", U.X.len(), 2);
          Err(SuperNovaError::NovaError(NovaError::ProofVerifyError))
        } else {
          Ok(())
        }
      })?;

    self.r_U_secondary[0].as_ref().map_or(Ok(()), |U| {
      if U.X.len() != 2 {
        debug!(
          "r_U_secondary got instance length {:?} != {:?}",
          U.X.len(),
          2
        );
        Err(SuperNovaError::NovaError(NovaError::ProofVerifyError))
      } else {
        Ok(())
      }
    })?;

    let num_field_primary_ro = 3 // params_next, i_new, program_counter_new
    + 2 * pp.F_arity_primary // zo, z1
    + (7 + 2 * pp.augmented_circuit_params_primary.get_n_limbs()); // # 1 * (7 + [X0, X1]*#num_limb)

    // secondary circuit
    let num_field_secondary_ro = 2 // params_next, i_new
    + 2 * pp.F_arity_secondary // zo, z1
    + self.num_augmented_circuits * (7 + 2 * pp.augmented_circuit_params_primary.get_n_limbs()); // #num_augmented_circuits * (7 + [X0, X1]*#num_limb)

    let (hash_primary, hash_secondary) = {
      let mut hasher = <G2 as Group>::RO::new(pp.ro_consts_secondary.clone(), num_field_primary_ro);
      hasher.absorb(self.pp_digest);
      hasher.absorb(G1::Scalar::from(self.i as u64));
      hasher.absorb(self.program_counter);

      for e in z0_primary {
        hasher.absorb(*e);
      }
      for e in &self.zi_primary {
        hasher.absorb(*e);
      }
      self.r_U_secondary[0].as_ref().map_or(
        Err(SuperNovaError::NovaError(NovaError::ProofVerifyError)),
        |U| {
          U.absorb_in_ro(&mut hasher);
          Ok(())
        },
      )?;

      let mut hasher2 =
        <G1 as Group>::RO::new(pp.ro_consts_primary.clone(), num_field_secondary_ro);
      hasher2.absorb(scalar_as_base::<G1>(self.pp_digest));
      hasher2.absorb(G2::Scalar::from(self.i as u64));

      for e in z0_secondary {
        hasher2.absorb(*e);
      }
      for e in &self.zi_secondary {
        hasher2.absorb(*e);
      }
      let default_value = RelaxedR1CSInstance::default(ck_primary, &pp.r1cs_shape_primary);
      self.r_U_primary.iter().for_each(|U| {
        U.as_ref()
          .unwrap_or(&default_value)
          .absorb_in_ro(&mut hasher2);
      });

      (
        hasher.squeeze(NUM_HASH_BITS),
        hasher2.squeeze(NUM_HASH_BITS),
      )
    };

    if hash_primary != self.l_u_secondary.X[0] {
      debug!(
        "hash_primary {:?} not equal l_u_secondary.X[0] {:?}",
        hash_primary, self.l_u_secondary.X[0]
      );
      return Err(SuperNovaError::NovaError(NovaError::ProofVerifyError));
    }
    if hash_secondary != scalar_as_base::<G2>(self.l_u_secondary.X[1]) {
      debug!(
        "hash_secondary {:?} not equal l_u_secondary.X[1] {:?}",
        hash_secondary, self.l_u_secondary.X[1]
      );
      return Err(SuperNovaError::NovaError(NovaError::ProofVerifyError));
    }

    // check the satisfiability of the provided `circuit_index` instance
    let default_instance = RelaxedR1CSInstance::default(ck_primary, &pp.r1cs_shape_primary);
    let default_witness = RelaxedR1CSWitness::default(&pp.r1cs_shape_primary);
    let (res_r_primary, (res_r_secondary, res_l_secondary)) = rayon::join(
      || {
        pp.r1cs_shape_primary.is_sat_relaxed(
          pp.ck_primary.as_ref().unwrap(),
          self.r_U_primary[claim.get_circuit_index()]
            .as_ref()
            .unwrap_or(&default_instance),
          self.r_W_primary[claim.get_circuit_index()]
            .as_ref()
            .unwrap_or(&default_witness),
        )
      },
      || {
        rayon::join(
          || {
            pp.r1cs_shape_secondary.is_sat_relaxed(
              pp.ck_secondary.as_ref().unwrap(),
              self.r_U_secondary[0].as_ref().unwrap(),
              self.r_W_secondary[0].as_ref().unwrap(),
            )
          },
          || {
            pp.r1cs_shape_secondary.is_sat(
              pp.ck_secondary.as_ref().unwrap(),
              &self.l_u_secondary,
              &self.l_w_secondary,
            )
          },
        )
      },
    );

    res_r_primary.map_err(|err| match err {
      NovaError::UnSatIndex(i) => SuperNovaError::UnSatIndex("r_primary", i),
      e => SuperNovaError::NovaError(e),
    })?;
    res_r_secondary.map_err(|err| match err {
      NovaError::UnSatIndex(i) => SuperNovaError::UnSatIndex("r_secondary", i),
      e => SuperNovaError::NovaError(e),
    })?;
    res_l_secondary.map_err(|err| match err {
      NovaError::UnSatIndex(i) => SuperNovaError::UnSatIndex("l_secondary", i),
      e => SuperNovaError::NovaError(e),
    })?;

    Ok(())
  }

  /// get program counter
  pub fn get_program_counter(&self) -> G1::Scalar {
    self.program_counter
  }
}

/// SuperNova helper trait, for implementors that provide sets of sub-circuits to be proved via NIVC. `C1` must be a
/// type (likely an `Enum`) for which a potentially-distinct instance can be supplied for each `index` below
/// `self.num_circuits()`.
pub trait NonUniformCircuit<G1, G2, C1>
where
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C1: EnforcingStepCircuit<G1::Scalar>,
{
  /// Initial program counter, defaults to zero.
  fn initial_program_counter(&self) -> G1::Scalar {
    G1::Scalar::ZERO
  }

  /// Initialize and return initial running claims.
  fn setup_running_claims(
    &self,
    running_claim_params: &RunningClaimParams<G1, G2, C1>,
  ) -> RunningClaims<G1, G2, C1, TrivialSecondaryCircuit<G2::Scalar>> {
    let running_claims = running_claim_params
      .pp_vec
      .iter()
      .enumerate()
      .map(|(i, pp)| {
        RunningClaim::new(
          pp,
          i,
          self.primary_circuit(i),
          self.secondary_circuit(),
          self.num_circuits(),
        )
      })
      .collect::<Vec<_>>();

    RunningClaims::new(running_claims)
  }

  /// How many circuits are provided?
  fn num_circuits(&self) -> usize;

  /// Return a new instance of the primary circuit at `index`.
  fn primary_circuit(&self, circuit_index: usize) -> C1;

  /// Return a new instance of the secondary circuit.
  fn secondary_circuit(&self) -> TrivialSecondaryCircuit<G2::Scalar> {
    Default::default()
  }
}

/// Compute the circuit digest of a supernova [StepCircuit].
pub fn circuit_digest<
  G1: Group<Base = <G2 as Group>::Scalar>,
  G2: Group<Base = <G1 as Group>::Scalar>,
  C: StepCircuit<G1::Scalar>,
>(
  circuit: &C,
  is_primary_circuit: bool,
  num_augmented_circuits: usize,
) -> G1::Scalar {
  let augmented_circuit_params =
    SuperNovaAugmentedCircuitParams::new(BN_LIMB_WIDTH, BN_N_LIMBS, is_primary_circuit);

  // ro_consts_circuit are parameterized by G2 because the type alias uses G2::Base = G1::Scalar
  let ro_consts_circuit: ROConstantsCircuit<G2> = ROConstantsCircuit::<G2>::default();

  // Initialize ck for the primary
  let augmented_circuit: SuperNovaAugmentedCircuit<'_, G2, C> = SuperNovaAugmentedCircuit::new(
    &augmented_circuit_params,
    None,
    circuit,
    ro_consts_circuit,
    num_augmented_circuits,
  );
  let mut cs: ShapeCS<G1> = ShapeCS::new();
  let _ = augmented_circuit.synthesize(&mut cs);
  cs.r1cs_shape().digest()
}
