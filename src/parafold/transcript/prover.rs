use ff::PrimeFieldBits;
use itertools::chain;
use neptune::sponge::api::{IOPattern, SpongeAPI, SpongeOp};
use neptune::sponge::vanilla::Mode::Simplex;
use neptune::sponge::vanilla::{Sponge, SpongeTrait};

use crate::parafold::transcript::{TranscriptConstants, TranscriptElement};
use crate::traits::CurveCycleEquipped;

#[derive(Clone, Debug)]
pub struct Transcript<E: CurveCycleEquipped> {
  constants: TranscriptConstants<E::Scalar>,

  // Initial carried-over state
  prev: Option<E::Scalar>,
  // Buffer of messages for the current round
  round_state: Vec<E::Scalar>,

  // Stores the entire set of prover messages, which can be deserialized by the circuit verifier.
  // Doesn't include the message from the current round, as these are stored in `round_state`
  // and copies into `buffer` after calling `squeeze`.
  buffer: Vec<TranscriptElement<E>>,
}

impl<E: CurveCycleEquipped> Transcript<E> {
  pub fn new(
    constants: TranscriptConstants<E::Scalar>,
    init: impl IntoIterator<Item = E::Scalar>,
  ) -> Self {
    Self {
      constants,
      prev: None,
      round_state: Vec::from_iter(init),
      buffer: vec![],
    }
  }

  pub fn absorb(&mut self, element: TranscriptElement<E>) {
    self.round_state.extend(element.to_field());
    self.buffer.push(element);
  }

  /// Adds a sequence of elements to the transcript, storing them for future deserialization by
  /// the corresponding verifier transcript.
  pub fn absorb_many(&mut self, elements: impl IntoIterator<Item = TranscriptElement<E>>) {
    for element in elements {
      self.absorb(element);
    }
  }

  // pub fn absorb_commitment_primary<E: Engine<Scalar = F>>(&mut self, c: Commitment<E>) {
  //   let limbs = commitment_to_hash_limbs::<E>(c);
  //   self.absorb_many(limbs);
  // }
  //
  // pub fn absorb_commitment_secondary<E: Engine<Base = F>>(&mut self, c: Commitment<E>) {
  //   let (x, y, _) = c.to_coordinates();
  //   self.absorb_many([x, y]);
  // }

  pub fn squeeze(&mut self) -> E::Scalar {
    let mut sponge = Sponge::new_with_constants(&self.constants, Simplex);

    let elements = chain!(self.prev.clone(), self.round_state.drain(..)).collect::<Vec<_>>();
    let num_absorbs = elements.len() as u32;

    let hash = {
      let acc = &mut ();
      let parameter = IOPattern(vec![SpongeOp::Absorb(num_absorbs), SpongeOp::Squeeze(1u32)]);
      sponge.start(parameter, None, acc);
      SpongeAPI::absorb(&mut sponge, num_absorbs, &elements, acc);
      let hash = SpongeAPI::squeeze(&mut sponge, 1, acc);
      sponge.finish(acc).unwrap();
      hash[0]
    };

    // save the current output
    self.prev = Some(hash);

    hash
  }

  pub fn squeeze_bits(&mut self, num_bits: usize) -> Vec<bool> {
    let hash = self.squeeze();
    hash.to_le_bits().into_iter().take(num_bits).collect()
  }

  pub fn seal(mut self) -> (E::Scalar, Vec<TranscriptElement<E>>) {
    if !self.round_state.is_empty() {
      let _ = self.squeeze();
    }
    (self.prev.unwrap(), self.buffer)
  }
}