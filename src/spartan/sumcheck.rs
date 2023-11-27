use crate::errors::NovaError;
use crate::spartan::polys::{
  multilinear::MultilinearPolynomial,
  univariate::{CompressedUniPoly, UniPoly},
};
use crate::traits::{Group, TranscriptEngineTrait};
use ff::Field;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub(crate) struct SumcheckProof<G: Group> {
  compressed_polys: Vec<CompressedUniPoly<G::Scalar>>,
}

impl<G: Group> SumcheckProof<G> {
  pub fn new(compressed_polys: Vec<CompressedUniPoly<G::Scalar>>) -> Self {
    Self { compressed_polys }
  }

  pub fn verify(
    &self,
    claim: G::Scalar,
    num_rounds: usize,
    degree_bound: usize,
    transcript: &mut G::TE,
  ) -> Result<(G::Scalar, Vec<G::Scalar>), NovaError> {
    let mut e = claim;
    let mut r: Vec<G::Scalar> = Vec::new();

    // verify that there is a univariate polynomial for each round
    if self.compressed_polys.len() != num_rounds {
      return Err(NovaError::InvalidSumcheckProof);
    }

    for i in 0..self.compressed_polys.len() {
      let poly = self.compressed_polys[i].decompress(&e);

      // verify degree bound
      if poly.degree() != degree_bound {
        return Err(NovaError::InvalidSumcheckProof);
      }

      // we do not need to check if poly(0) + poly(1) = e, as
      // decompress() call above already ensures that holds
      debug_assert_eq!(poly.eval_at_zero() + poly.eval_at_one(), e);

      // append the prover's message to the transcript
      transcript.absorb(b"p", &poly);

      //derive the verifier's challenge for the next round
      let r_i = transcript.squeeze(b"c")?;

      r.push(r_i);

      // evaluate the claimed degree-ell polynomial at r_i
      e = poly.evaluate(&r_i);
    }

    Ok((e, r))
  }

  pub fn verify_batch(
    &self,
    claims: &[G::Scalar],
    num_rounds: &[usize],
    coeffs: &[G::Scalar],
    degree_bound: usize,
    transcript: &mut G::TE,
  ) -> Result<(G::Scalar, Vec<G::Scalar>), NovaError> {
    let num_instances = claims.len();
    assert_eq!(num_rounds.len(), num_instances);
    assert_eq!(coeffs.len(), num_instances);

    // n = maxᵢ{nᵢ}
    let num_rounds_max = num_rounds.iter().cloned().max().unwrap();

    // Random linear combination of claims,
    // where each claim is scaled by 2^{n-nᵢ} to account for the padding.
    //
    // claim = ∑ᵢ coeffᵢ⋅2^{n-nᵢ}⋅cᵢ
    let claim = claims
      .iter()
      .zip(num_rounds.iter())
      .map(|(claim, num_rounds)| {
        let scaling_factor = 1 << (num_rounds_max - num_rounds);
        G::Scalar::from(scaling_factor as u64) * claim
      })
      .zip(coeffs.iter())
      .map(|(scaled_claim, coeff)| scaled_claim * coeff)
      .sum();

    self.verify(claim, num_rounds_max, degree_bound, transcript)
  }

  #[inline]
  pub(in crate::spartan) fn compute_eval_points_quad<F>(
    poly_A: &MultilinearPolynomial<G::Scalar>,
    poly_B: &MultilinearPolynomial<G::Scalar>,
    comb_func: &F,
  ) -> (G::Scalar, G::Scalar)
  where
    F: Fn(&G::Scalar, &G::Scalar) -> G::Scalar + Sync,
  {
    let len = poly_A.len() / 2;
    (0..len)
      .into_par_iter()
      .map(|i| {
        // eval 0: bound_func is A(low)
        let eval_point_0 = comb_func(&poly_A[i], &poly_B[i]);

        // eval 2: bound_func is -A(low) + 2*A(high)
        let poly_A_bound_point = poly_A[len + i] + poly_A[len + i] - poly_A[i];
        let poly_B_bound_point = poly_B[len + i] + poly_B[len + i] - poly_B[i];
        let eval_point_2 = comb_func(&poly_A_bound_point, &poly_B_bound_point);
        (eval_point_0, eval_point_2)
      })
      .reduce(
        || (G::Scalar::ZERO, G::Scalar::ZERO),
        |a, b| (a.0 + b.0, a.1 + b.1),
      )
  }

  pub fn prove_quad<F>(
    claim: &G::Scalar,
    num_rounds: usize,
    poly_A: &mut MultilinearPolynomial<G::Scalar>,
    poly_B: &mut MultilinearPolynomial<G::Scalar>,
    comb_func: F,
    transcript: &mut G::TE,
  ) -> Result<(Self, Vec<G::Scalar>, Vec<G::Scalar>), NovaError>
  where
    F: Fn(&G::Scalar, &G::Scalar) -> G::Scalar + Sync,
  {
    let mut r: Vec<G::Scalar> = Vec::new();
    let mut polys: Vec<CompressedUniPoly<G::Scalar>> = Vec::new();
    let mut claim_per_round = *claim;
    for _ in 0..num_rounds {
      let poly = {
        let (eval_point_0, eval_point_2) =
          Self::compute_eval_points_quad(poly_A, poly_B, &comb_func);

        let evals = vec![eval_point_0, claim_per_round - eval_point_0, eval_point_2];
        UniPoly::from_evals(&evals)
      };

      // append the prover's message to the transcript
      transcript.absorb(b"p", &poly);

      //derive the verifier's challenge for the next round
      let r_i = transcript.squeeze(b"c")?;
      r.push(r_i);
      polys.push(poly.compress());

      // Set up next round
      claim_per_round = poly.evaluate(&r_i);

      // bind all tables to the verifier's challenge
      rayon::join(
        || poly_A.bind_poly_var_top(&r_i),
        || poly_B.bind_poly_var_top(&r_i),
      );
    }

    Ok((
      SumcheckProof {
        compressed_polys: polys,
      },
      r,
      vec![poly_A[0], poly_B[0]],
    ))
  }

  pub fn prove_quad_batch<F>(
    claims: &[G::Scalar],
    num_rounds: &[usize],
    mut poly_A_vec: Vec<MultilinearPolynomial<G::Scalar>>,
    mut poly_B_vec: Vec<MultilinearPolynomial<G::Scalar>>,
    coeffs: &[G::Scalar],
    comb_func: F,
    transcript: &mut G::TE,
  ) -> Result<(Self, Vec<G::Scalar>, (Vec<G::Scalar>, Vec<G::Scalar>)), NovaError>
  where
    F: Fn(&G::Scalar, &G::Scalar) -> G::Scalar + Sync,
  {
    let num_claims = claims.len();

    assert_eq!(num_rounds.len(), num_claims);
    assert_eq!(poly_A_vec.len(), num_claims);
    assert_eq!(poly_B_vec.len(), num_claims);
    assert_eq!(coeffs.len(), num_claims);

    num_rounds
      .iter()
      .zip(poly_A_vec.iter().zip(poly_B_vec.iter()))
      .for_each(|(num_rounds, (poly_A, poly_B))| {
        let poly_size = 1 << num_rounds;
        assert_eq!(poly_A.len(), poly_size);
        assert_eq!(poly_B.len(), poly_size);
      });

    let num_rounds_max = num_rounds.iter().cloned().max().unwrap();
    let mut e = claims
      .iter()
      .zip(num_rounds)
      .map(|(claim, num_rounds)| {
        G::Scalar::from((1 << (num_rounds_max - num_rounds)) as u64) * claim
      })
      .zip(coeffs)
      .map(|(claim, c)| claim * c)
      .sum();
    let mut r: Vec<G::Scalar> = Vec::new();
    let mut quad_polys: Vec<CompressedUniPoly<G::Scalar>> = Vec::new();

    for current_round in 0..num_rounds_max {
      let remaining_rounds = num_rounds_max - current_round;
      let evals: Vec<(G::Scalar, G::Scalar)> = num_rounds
        .par_iter()
        .zip(claims.par_iter())
        .zip(poly_A_vec.par_iter().zip(poly_B_vec.par_iter()))
        .map(|((&num_rounds, claim), (poly_A, poly_B))| {
          if remaining_rounds <= num_rounds {
            Self::compute_eval_points_quad(poly_A, poly_B, &comb_func)
          } else {
            let remaining_variables = remaining_rounds - num_rounds - 1;
            let scaled_claim = G::Scalar::from((1 << remaining_variables) as u64) * claim;
            (scaled_claim, scaled_claim)
          }
        })
        .collect();

      let evals_combined_0 = (0..evals.len()).map(|i| evals[i].0 * coeffs[i]).sum();
      let evals_combined_2 = (0..evals.len()).map(|i| evals[i].1 * coeffs[i]).sum();

      let evals = vec![evals_combined_0, e - evals_combined_0, evals_combined_2];
      let poly = UniPoly::from_evals(&evals);

      // append the prover's message to the transcript
      transcript.absorb(b"p", &poly);

      // derive the verifier's challenge for the next round
      let r_i = transcript.squeeze(b"c")?;
      r.push(r_i);

      // bound all tables to the verifier's challenge
      num_rounds
        .par_iter()
        .zip(poly_A_vec.par_iter_mut())
        .zip(poly_B_vec.par_iter_mut())
        .for_each(|((&num_rounds, poly_A), poly_B)| {
          if remaining_rounds <= num_rounds {
            let _ = rayon::join(
              || poly_A.bind_poly_var_top(&r_i),
              || poly_B.bind_poly_var_top(&r_i),
            );
          }
        });

      e = poly.evaluate(&r_i);
      quad_polys.push(poly.compress());
    }
    poly_A_vec.iter().for_each(|p| assert_eq!(p.len(), 1));
    poly_B_vec.iter().for_each(|p| assert_eq!(p.len(), 1));

    let poly_A_final = poly_A_vec
      .into_iter()
      .map(|poly| poly[0])
      .collect::<Vec<_>>();
    let poly_B_final = poly_B_vec
      .into_iter()
      .map(|poly| poly[0])
      .collect::<Vec<_>>();

    let eval_expected = poly_A_final
      .iter()
      .zip(poly_B_final.iter())
      .map(|(eA, eB)| comb_func(eA, eB))
      .zip(coeffs.iter())
      .map(|(e, c)| e * c)
      .sum::<G::Scalar>();
    assert_eq!(e, eval_expected);

    let claims_prod = (poly_A_final, poly_B_final);

    Ok((SumcheckProof::new(quad_polys), r, claims_prod))
  }

  #[inline]
  pub(in crate::spartan) fn compute_eval_points_cubic<F>(
    poly_A: &MultilinearPolynomial<G::Scalar>,
    poly_B: &MultilinearPolynomial<G::Scalar>,
    poly_C: &MultilinearPolynomial<G::Scalar>,
    comb_func: &F,
  ) -> (G::Scalar, G::Scalar, G::Scalar)
  where
    F: Fn(&G::Scalar, &G::Scalar, &G::Scalar) -> G::Scalar + Sync,
  {
    let len = poly_A.len() / 2;
    (0..len)
      .into_par_iter()
      .map(|i| {
        // eval 0: bound_func is A(low)
        let eval_point_0 = comb_func(&poly_A[i], &poly_B[i], &poly_C[i]);

        // eval 2: bound_func is -A(low) + 2*A(high)
        let poly_A_bound_point = poly_A[len + i] + poly_A[len + i] - poly_A[i];
        let poly_B_bound_point = poly_B[len + i] + poly_B[len + i] - poly_B[i];
        let poly_C_bound_point = poly_C[len + i] + poly_C[len + i] - poly_C[i];
        let eval_point_2 = comb_func(
          &poly_A_bound_point,
          &poly_B_bound_point,
          &poly_C_bound_point,
        );

        // eval 3: bound_func is -2A(low) + 3A(high); computed incrementally with bound_func applied to eval(2)
        let poly_A_bound_point = poly_A_bound_point + poly_A[len + i] - poly_A[i];
        let poly_B_bound_point = poly_B_bound_point + poly_B[len + i] - poly_B[i];
        let poly_C_bound_point = poly_C_bound_point + poly_C[len + i] - poly_C[i];
        let eval_point_3 = comb_func(
          &poly_A_bound_point,
          &poly_B_bound_point,
          &poly_C_bound_point,
        );
        (eval_point_0, eval_point_2, eval_point_3)
      })
      .reduce(
        || (G::Scalar::ZERO, G::Scalar::ZERO, G::Scalar::ZERO),
        |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2),
      )
  }

  #[inline]
  pub(in crate::spartan) fn compute_eval_points_cubic_with_additive_term<F>(
    poly_A: &MultilinearPolynomial<G::Scalar>,
    poly_B: &MultilinearPolynomial<G::Scalar>,
    poly_C: &MultilinearPolynomial<G::Scalar>,
    poly_D: &MultilinearPolynomial<G::Scalar>,
    comb_func: &F,
  ) -> (G::Scalar, G::Scalar, G::Scalar)
  where
    F: Fn(&G::Scalar, &G::Scalar, &G::Scalar, &G::Scalar) -> G::Scalar + Sync,
  {
    let len = poly_A.len() / 2;
    (0..len)
      .into_par_iter()
      .map(|i| {
        // eval 0: bound_func is A(low)
        let eval_point_0 = comb_func(&poly_A[i], &poly_B[i], &poly_C[i], &poly_D[i]);

        // eval 2: bound_func is -A(low) + 2*A(high)
        let poly_A_bound_point = poly_A[len + i] + poly_A[len + i] - poly_A[i];
        let poly_B_bound_point = poly_B[len + i] + poly_B[len + i] - poly_B[i];
        let poly_C_bound_point = poly_C[len + i] + poly_C[len + i] - poly_C[i];
        let poly_D_bound_point = poly_D[len + i] + poly_D[len + i] - poly_D[i];
        let eval_point_2 = comb_func(
          &poly_A_bound_point,
          &poly_B_bound_point,
          &poly_C_bound_point,
          &poly_D_bound_point,
        );

        // eval 3: bound_func is -2A(low) + 3A(high); computed incrementally with bound_func applied to eval(2)
        let poly_A_bound_point = poly_A_bound_point + poly_A[len + i] - poly_A[i];
        let poly_B_bound_point = poly_B_bound_point + poly_B[len + i] - poly_B[i];
        let poly_C_bound_point = poly_C_bound_point + poly_C[len + i] - poly_C[i];
        let poly_D_bound_point = poly_D_bound_point + poly_D[len + i] - poly_D[i];
        let eval_point_3 = comb_func(
          &poly_A_bound_point,
          &poly_B_bound_point,
          &poly_C_bound_point,
          &poly_D_bound_point,
        );
        (eval_point_0, eval_point_2, eval_point_3)
      })
      .reduce(
        || (G::Scalar::ZERO, G::Scalar::ZERO, G::Scalar::ZERO),
        |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2),
      )
  }

  pub fn prove_cubic_with_additive_term<F>(
    claim: &G::Scalar,
    num_rounds: usize,
    poly_A: &mut MultilinearPolynomial<G::Scalar>,
    poly_B: &mut MultilinearPolynomial<G::Scalar>,
    poly_C: &mut MultilinearPolynomial<G::Scalar>,
    poly_D: &mut MultilinearPolynomial<G::Scalar>,
    comb_func: F,
    transcript: &mut G::TE,
  ) -> Result<(Self, Vec<G::Scalar>, Vec<G::Scalar>), NovaError>
  where
    F: Fn(&G::Scalar, &G::Scalar, &G::Scalar, &G::Scalar) -> G::Scalar + Sync,
  {
    let mut r: Vec<G::Scalar> = Vec::new();
    let mut polys: Vec<CompressedUniPoly<G::Scalar>> = Vec::new();
    let mut claim_per_round = *claim;

    for _ in 0..num_rounds {
      let poly = {
        // Make an iterator returning the contributions to the evaluations
        let (eval_point_0, eval_point_2, eval_point_3) =
          Self::compute_eval_points_cubic_with_additive_term(
            poly_A, poly_B, poly_C, poly_D, &comb_func,
          );

        let evals = vec![
          eval_point_0,
          claim_per_round - eval_point_0,
          eval_point_2,
          eval_point_3,
        ];
        UniPoly::from_evals(&evals)
      };

      // append the prover's message to the transcript
      transcript.absorb(b"p", &poly);

      //derive the verifier's challenge for the next round
      let r_i = transcript.squeeze(b"c")?;
      r.push(r_i);
      polys.push(poly.compress());

      // Set up next round
      claim_per_round = poly.evaluate(&r_i);

      // bound all tables to the verifier's challenge
      rayon::join(
        || {
          rayon::join(
            || poly_A.bind_poly_var_top(&r_i),
            || poly_B.bind_poly_var_top(&r_i),
          )
        },
        || {
          rayon::join(
            || poly_C.bind_poly_var_top(&r_i),
            || poly_D.bind_poly_var_top(&r_i),
          )
        },
      );
    }

    Ok((
      SumcheckProof {
        compressed_polys: polys,
      },
      r,
      vec![poly_A[0], poly_B[0], poly_C[0], poly_D[0]],
    ))
  }

  pub fn prove_cubic_with_additive_term_batch<F>(
    claims: &[G::Scalar],
    num_rounds: &[usize],
    mut poly_A_vec: Vec<MultilinearPolynomial<G::Scalar>>,
    mut poly_B_vec: Vec<MultilinearPolynomial<G::Scalar>>,
    mut poly_C_vec: Vec<MultilinearPolynomial<G::Scalar>>,
    mut poly_D_vec: Vec<MultilinearPolynomial<G::Scalar>>,
    coeffs: &[G::Scalar],
    comb_func: F,
    transcript: &mut G::TE,
  ) -> Result<(Self, Vec<G::Scalar>, Vec<Vec<G::Scalar>>), NovaError>
  where
    F: Fn(&G::Scalar, &G::Scalar, &G::Scalar, &G::Scalar) -> G::Scalar + Sync,
  {
    let num_instances = claims.len();
    assert_eq!(num_rounds.len(), num_instances);
    assert_eq!(coeffs.len(), num_instances);
    assert_eq!(poly_A_vec.len(), num_instances);
    assert_eq!(poly_B_vec.len(), num_instances);
    assert_eq!(poly_C_vec.len(), num_instances);
    assert_eq!(poly_D_vec.len(), num_instances);

    num_rounds
      .iter()
      .zip(poly_A_vec.iter())
      .zip(poly_B_vec.iter())
      .zip(poly_C_vec.iter())
      .zip(poly_D_vec.iter())
      .for_each(|((((num_rounds, a), b), c), d)| {
        let expected_size = 1 << num_rounds;
        assert_eq!(a.len(), expected_size);
        assert_eq!(b.len(), expected_size);
        assert_eq!(c.len(), expected_size);
        assert_eq!(d.len(), expected_size);
      });
    let num_rounds_max = num_rounds.iter().cloned().max().unwrap();

    let mut r: Vec<G::Scalar> = Vec::new();
    let mut polys: Vec<CompressedUniPoly<G::Scalar>> = Vec::new();
    let mut claim_per_round = claims
      .iter()
      .zip(num_rounds)
      .map(|(claim, num_rounds)| {
        G::Scalar::from((1 << (num_rounds_max - num_rounds)) as u64) * claim
      })
      .zip(coeffs.iter())
      .map(|(claim, c)| claim * c)
      .sum();

    for current_round in 0..num_rounds_max {
      let remaining_rounds = num_rounds_max - current_round;
      let evals: Vec<(G::Scalar, G::Scalar, G::Scalar)> = num_rounds
        .par_iter()
        .zip(claims.par_iter())
        .zip(poly_A_vec.par_iter())
        .zip(poly_B_vec.par_iter())
        .zip(poly_C_vec.par_iter())
        .zip(poly_D_vec.par_iter())
        .map(
          |(((((&num_rounds, claim), poly_A), poly_B), poly_C), poly_D)| {
            if remaining_rounds <= num_rounds {
              Self::compute_eval_points_cubic_with_additive_term(
                poly_A, poly_B, poly_C, poly_D, &comb_func,
              )
            } else {
              let remaining_variables = remaining_rounds - num_rounds - 1;
              let scaled_claim = G::Scalar::from((1 << remaining_variables) as u64) * claim;
              (scaled_claim, scaled_claim, scaled_claim)
            }
          },
        )
        .collect();

      let evals_combined_0 = (0..num_instances).map(|i| evals[i].0 * coeffs[i]).sum();
      let evals_combined_2 = (0..num_instances).map(|i| evals[i].1 * coeffs[i]).sum();
      let evals_combined_3 = (0..num_instances).map(|i| evals[i].2 * coeffs[i]).sum();

      let evals = vec![
        evals_combined_0,
        claim_per_round - evals_combined_0,
        evals_combined_2,
        evals_combined_3,
      ];
      let poly = UniPoly::from_evals(&evals);

      // append the prover's message to the transcript
      transcript.absorb(b"p", &poly);

      //derive the verifier's challenge for the next round
      let r_i = transcript.squeeze(b"c")?;
      r.push(r_i);

      polys.push(poly.compress());

      // Set up next round
      claim_per_round = poly.evaluate(&r_i);

      // bound all the tables to the verifier's challenge
      num_rounds
        .par_iter()
        .zip(poly_A_vec.par_iter_mut())
        .zip(poly_B_vec.par_iter_mut())
        .zip(poly_C_vec.par_iter_mut())
        .zip(poly_D_vec.par_iter_mut())
        .for_each(|((((&num_rounds, poly_A), poly_B), poly_C), poly_D)| {
          if remaining_rounds <= num_rounds {
            let _ = rayon::join(
              || {
                rayon::join(
                  || poly_A.bind_poly_var_top(&r_i),
                  || poly_B.bind_poly_var_top(&r_i),
                )
              },
              || {
                rayon::join(
                  || poly_C.bind_poly_var_top(&r_i),
                  || poly_D.bind_poly_var_top(&r_i),
                )
              },
            );
          }
        });
    }

    let poly_A_final = poly_A_vec.into_iter().map(|poly| poly[0]).collect();
    let poly_B_final = poly_B_vec.into_iter().map(|poly| poly[0]).collect();
    let poly_C_final = poly_C_vec.into_iter().map(|poly| poly[0]).collect();
    let poly_D_final = poly_D_vec.into_iter().map(|poly| poly[0]).collect();

    Ok((
      SumcheckProof {
        compressed_polys: polys,
      },
      r,
      vec![poly_A_final, poly_B_final, poly_C_final, poly_D_final],
    ))
  }
}
