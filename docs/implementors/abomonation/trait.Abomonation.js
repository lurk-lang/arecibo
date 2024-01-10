(function() {var implementors = {
"arecibo":[["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt; Abomonation for <a class=\"struct\" href=\"arecibo/provider/ipa_pc/struct.ProverKey.html\" title=\"struct arecibo::provider::ipa_pc::ProverKey\">ProverKey</a>&lt;E&gt;"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>, EE: <a class=\"trait\" href=\"arecibo/traits/evaluation/trait.EvaluationEngineTrait.html\" title=\"trait arecibo::traits::evaluation::EvaluationEngineTrait\">EvaluationEngineTrait</a>&lt;E&gt;&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/batched_ppsnark/struct.VerifierKey.html\" title=\"struct arecibo::spartan::batched_ppsnark::VerifierKey\">VerifierKey</a>&lt;E, EE&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>, EE: <a class=\"trait\" href=\"arecibo/traits/evaluation/trait.EvaluationEngineTrait.html\" title=\"trait arecibo::traits::evaluation::EvaluationEngineTrait\">EvaluationEngineTrait</a>&lt;E&gt;&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/snark/struct.ProverKey.html\" title=\"struct arecibo::spartan::snark::ProverKey\">ProverKey</a>&lt;E, EE&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E1, E2, C1, C2&gt; Abomonation for <a class=\"struct\" href=\"arecibo/struct.PublicParams.html\" title=\"struct arecibo::PublicParams\">PublicParams</a>&lt;E1, E2, C1, C2&gt;<span class=\"where fmt-newline\">where\n    E1: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E2 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    E2: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E1 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C1: <a class=\"trait\" href=\"arecibo/traits/circuit/trait.StepCircuit.html\" title=\"trait arecibo::traits::circuit::StepCircuit\">StepCircuit</a>&lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C2: <a class=\"trait\" href=\"arecibo/traits/circuit/trait.StepCircuit.html\" title=\"trait arecibo::traits::circuit::StepCircuit\">StepCircuit</a>&lt;E2::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    &lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,\n    &lt;E2::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: Engine&gt; Abomonation for <a class=\"struct\" href=\"arecibo/provider/non_hiding_zeromorph/struct.ZMProverKey.html\" title=\"struct arecibo::provider::non_hiding_zeromorph::ZMProverKey\">ZMProverKey</a>&lt;E&gt;"],["impl&lt;E: Engine&gt; Abomonation for <a class=\"struct\" href=\"arecibo/provider/non_hiding_zeromorph/struct.ZMVerifierKey.html\" title=\"struct arecibo::provider::non_hiding_zeromorph::ZMVerifierKey\">ZMVerifierKey</a>&lt;E&gt;"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>, EE: <a class=\"trait\" href=\"arecibo/traits/evaluation/trait.EvaluationEngineTrait.html\" title=\"trait arecibo::traits::evaluation::EvaluationEngineTrait\">EvaluationEngineTrait</a>&lt;E&gt;&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/ppsnark/struct.VerifierKey.html\" title=\"struct arecibo::spartan::ppsnark::VerifierKey\">VerifierKey</a>&lt;E, EE&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E1, E2&gt; Abomonation for <a class=\"struct\" href=\"arecibo/supernova/struct.AuxParams.html\" title=\"struct arecibo::supernova::AuxParams\">AuxParams</a>&lt;E1, E2&gt;<span class=\"where fmt-newline\">where\n    E1: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E2 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    E2: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E1 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    &lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,\n    &lt;E2::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E1, E2, C1, C2, S1, S2&gt; Abomonation for <a class=\"struct\" href=\"arecibo/supernova/snark/struct.ProverKey.html\" title=\"struct arecibo::supernova::snark::ProverKey\">ProverKey</a>&lt;E1, E2, C1, C2, S1, S2&gt;<span class=\"where fmt-newline\">where\n    E1: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E2 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    E2: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E1 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C1: <a class=\"trait\" href=\"arecibo/supernova/trait.StepCircuit.html\" title=\"trait arecibo::supernova::StepCircuit\">StepCircuit</a>&lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C2: <a class=\"trait\" href=\"arecibo/supernova/trait.StepCircuit.html\" title=\"trait arecibo::supernova::StepCircuit\">StepCircuit</a>&lt;E2::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    S1: <a class=\"trait\" href=\"arecibo/traits/snark/trait.BatchedRelaxedR1CSSNARKTrait.html\" title=\"trait arecibo::traits::snark::BatchedRelaxedR1CSSNARKTrait\">BatchedRelaxedR1CSSNARKTrait</a>&lt;E1&gt;,\n    S2: <a class=\"trait\" href=\"arecibo/traits/snark/trait.RelaxedR1CSSNARKTrait.html\" title=\"trait arecibo::traits::snark::RelaxedR1CSSNARKTrait\">RelaxedR1CSSNARKTrait</a>&lt;E2&gt;,\n    &lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt; Abomonation for <a class=\"struct\" href=\"arecibo/struct.R1CSWithArity.html\" title=\"struct arecibo::R1CSWithArity\">R1CSWithArity</a>&lt;E&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/ppsnark/struct.R1CSShapeSparkCommitment.html\" title=\"struct arecibo::spartan::ppsnark::R1CSShapeSparkCommitment\">R1CSShapeSparkCommitment</a>&lt;E&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt; Abomonation for <a class=\"struct\" href=\"arecibo/provider/ipa_pc/struct.VerifierKey.html\" title=\"struct arecibo::provider::ipa_pc::VerifierKey\">VerifierKey</a>&lt;E&gt;"],["impl&lt;E1, E2, C1, C2, S1, S2&gt; Abomonation for <a class=\"struct\" href=\"arecibo/struct.ProverKey.html\" title=\"struct arecibo::ProverKey\">ProverKey</a>&lt;E1, E2, C1, C2, S1, S2&gt;<span class=\"where fmt-newline\">where\n    E1: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E2 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    E2: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E1 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C1: <a class=\"trait\" href=\"arecibo/traits/circuit/trait.StepCircuit.html\" title=\"trait arecibo::traits::circuit::StepCircuit\">StepCircuit</a>&lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C2: <a class=\"trait\" href=\"arecibo/traits/circuit/trait.StepCircuit.html\" title=\"trait arecibo::traits::circuit::StepCircuit\">StepCircuit</a>&lt;E2::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    S1: <a class=\"trait\" href=\"arecibo/traits/snark/trait.RelaxedR1CSSNARKTrait.html\" title=\"trait arecibo::traits::snark::RelaxedR1CSSNARKTrait\">RelaxedR1CSSNARKTrait</a>&lt;E1&gt;,\n    S2: <a class=\"trait\" href=\"arecibo/traits/snark/trait.RelaxedR1CSSNARKTrait.html\" title=\"trait arecibo::traits::snark::RelaxedR1CSSNARKTrait\">RelaxedR1CSSNARKTrait</a>&lt;E2&gt;,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>, EE: <a class=\"trait\" href=\"arecibo/traits/evaluation/trait.EvaluationEngineTrait.html\" title=\"trait arecibo::traits::evaluation::EvaluationEngineTrait\">EvaluationEngineTrait</a>&lt;E&gt;&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/batched_ppsnark/struct.ProverKey.html\" title=\"struct arecibo::spartan::batched_ppsnark::ProverKey\">ProverKey</a>&lt;E, EE&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E1, E2, C1, C2, S1, S2&gt; Abomonation for <a class=\"struct\" href=\"arecibo/supernova/snark/struct.VerifierKey.html\" title=\"struct arecibo::supernova::snark::VerifierKey\">VerifierKey</a>&lt;E1, E2, C1, C2, S1, S2&gt;<span class=\"where fmt-newline\">where\n    E1: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E2 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    E2: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E1 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C1: <a class=\"trait\" href=\"arecibo/supernova/trait.StepCircuit.html\" title=\"trait arecibo::supernova::StepCircuit\">StepCircuit</a>&lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C2: <a class=\"trait\" href=\"arecibo/supernova/trait.StepCircuit.html\" title=\"trait arecibo::supernova::StepCircuit\">StepCircuit</a>&lt;E2::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    S1: <a class=\"trait\" href=\"arecibo/traits/snark/trait.BatchedRelaxedR1CSSNARKTrait.html\" title=\"trait arecibo::traits::snark::BatchedRelaxedR1CSSNARKTrait\">BatchedRelaxedR1CSSNARKTrait</a>&lt;E1&gt;,\n    S2: <a class=\"trait\" href=\"arecibo/traits/snark/trait.RelaxedR1CSSNARKTrait.html\" title=\"trait arecibo::traits::snark::RelaxedR1CSSNARKTrait\">RelaxedR1CSSNARKTrait</a>&lt;E2&gt;,\n    &lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>, EE: <a class=\"trait\" href=\"arecibo/traits/evaluation/trait.EvaluationEngineTrait.html\" title=\"trait arecibo::traits::evaluation::EvaluationEngineTrait\">EvaluationEngineTrait</a>&lt;E&gt;&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/batched/struct.ProverKey.html\" title=\"struct arecibo::spartan::batched::ProverKey\">ProverKey</a>&lt;E, EE&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>, EE: <a class=\"trait\" href=\"arecibo/traits/evaluation/trait.EvaluationEngineTrait.html\" title=\"trait arecibo::traits::evaluation::EvaluationEngineTrait\">EvaluationEngineTrait</a>&lt;E&gt;&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/batched/struct.VerifierKey.html\" title=\"struct arecibo::spartan::batched::VerifierKey\">VerifierKey</a>&lt;E, EE&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt; Abomonation for <a class=\"struct\" href=\"arecibo/r1cs/struct.R1CSShape.html\" title=\"struct arecibo::r1cs::R1CSShape\">R1CSShape</a>&lt;E&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E1, E2, C1, C2, S1, S2&gt; Abomonation for <a class=\"struct\" href=\"arecibo/struct.VerifierKey.html\" title=\"struct arecibo::VerifierKey\">VerifierKey</a>&lt;E1, E2, C1, C2, S1, S2&gt;<span class=\"where fmt-newline\">where\n    E1: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E2 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    E2: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&lt;Base = &lt;E1 as <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt;::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C1: <a class=\"trait\" href=\"arecibo/traits/circuit/trait.StepCircuit.html\" title=\"trait arecibo::traits::circuit::StepCircuit\">StepCircuit</a>&lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    C2: <a class=\"trait\" href=\"arecibo/traits/circuit/trait.StepCircuit.html\" title=\"trait arecibo::traits::circuit::StepCircuit\">StepCircuit</a>&lt;E2::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a>&gt;,\n    S1: <a class=\"trait\" href=\"arecibo/traits/snark/trait.RelaxedR1CSSNARKTrait.html\" title=\"trait arecibo::traits::snark::RelaxedR1CSSNARKTrait\">RelaxedR1CSSNARKTrait</a>&lt;E1&gt;,\n    S2: <a class=\"trait\" href=\"arecibo/traits/snark/trait.RelaxedR1CSSNARKTrait.html\" title=\"trait arecibo::traits::snark::RelaxedR1CSSNARKTrait\">RelaxedR1CSSNARKTrait</a>&lt;E2&gt;,\n    &lt;E1::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>, EE: <a class=\"trait\" href=\"arecibo/traits/evaluation/trait.EvaluationEngineTrait.html\" title=\"trait arecibo::traits::evaluation::EvaluationEngineTrait\">EvaluationEngineTrait</a>&lt;E&gt;&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/ppsnark/struct.ProverKey.html\" title=\"struct arecibo::spartan::ppsnark::ProverKey\">ProverKey</a>&lt;E, EE&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/ppsnark/struct.R1CSShapeSparkRepr.html\" title=\"struct arecibo::spartan::ppsnark::R1CSShapeSparkRepr\">R1CSShapeSparkRepr</a>&lt;E&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"],["impl&lt;E: <a class=\"trait\" href=\"arecibo/traits/trait.Engine.html\" title=\"trait arecibo::traits::Engine\">Engine</a>, EE: <a class=\"trait\" href=\"arecibo/traits/evaluation/trait.EvaluationEngineTrait.html\" title=\"trait arecibo::traits::evaluation::EvaluationEngineTrait\">EvaluationEngineTrait</a>&lt;E&gt;&gt; Abomonation for <a class=\"struct\" href=\"arecibo/spartan/snark/struct.VerifierKey.html\" title=\"struct arecibo::spartan::snark::VerifierKey\">VerifierKey</a>&lt;E, EE&gt;<span class=\"where fmt-newline\">where\n    &lt;E::<a class=\"associatedtype\" href=\"arecibo/traits/trait.Engine.html#associatedtype.Scalar\" title=\"type arecibo::traits::Engine::Scalar\">Scalar</a> as PrimeField&gt;::Repr: Abomonation,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()