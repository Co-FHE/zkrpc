mod traits;
/// Prove knowing knowledge of six private inputs: x1 x2 x3 a b c
/// s.t: x1a + x2b + x3c = out
use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Constraints, Error, Instance, Selector},
    poly::Rotation,
};
use halo2curves::serde::SerdeObject;
pub use traits::*;
// use halo2_proofs::{dev::MockProver, pasta::Fp};

/// Circuit design:
/// | ins   | a0    | a1    | s_mul | s_add |
/// |-------|-------|-------|-------|-------|
/// | out   |    a  |       |       |       |
/// |       |    b  |       |       |       |
/// |       |    c  |       |       |       |

#[derive(Debug, Clone)]
struct CircuitConfig {
    instance: Column<Instance>,
    advice: [Column<Advice>; 2],
    s_mul: Selector,
    s_add: Selector,
}

#[derive(Clone)]
struct Number<F: Field>(AssignedCell<F, F>);

#[derive(Default)]
struct MyCircuit<F: Field> {
    // constant: F,
    a: Value<F>,
    b: Value<F>,
    c: Value<F>,
    x1: Value<F>,
    x2: Value<F>,
    x3: Value<F>,
}

fn load_private<F: Field>(
    config: &CircuitConfig,
    mut layouter: impl Layouter<F>,
    value: Value<F>,
) -> Result<Number<F>, Error> {
    layouter.assign_region(
        || "load private",
        |mut region| {
            region
                .assign_advice(|| "private input", config.advice[0], 0, || value)
                .map(Number)
        },
    )
}

#[allow(unused)]
fn load_constant<F: Field>(
    config: &CircuitConfig,
    mut layouter: impl Layouter<F>,
    constant: F,
) -> Result<Number<F>, Error> {
    layouter.assign_region(
        || "load private",
        |mut region| {
            region
                .assign_advice_from_constant(|| "private input", config.advice[0], 0, constant)
                .map(Number)
        },
    )
}

fn mul<F: Field>(
    config: &CircuitConfig,
    mut layouter: impl Layouter<F>,
    a: Number<F>,
    b: Number<F>,
) -> Result<Number<F>, Error> {
    layouter.assign_region(
        || "mul",
        |mut region| {
            config.s_mul.enable(&mut region, 0)?;
            a.0.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;
            b.0.copy_advice(|| "rhs", &mut region, config.advice[1], 0)?;

            let value = a.0.value().copied() * b.0.value().copied();
            region
                .assign_advice(|| "res=lhs*rhs", config.advice[0], 1, || value)
                .map(Number)
        },
    )
}

fn add<F: Field>(
    config: &CircuitConfig,
    mut layouter: impl Layouter<F>,
    a: Number<F>,
    b: Number<F>,
) -> Result<Number<F>, Error> {
    layouter.assign_region(
        || "add",
        |mut region| {
            config.s_add.enable(&mut region, 0)?;
            a.0.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;
            b.0.copy_advice(|| "rhs", &mut region, config.advice[1], 0)?;

            let value = a.0.value().copied() + b.0.value().copied();
            region
                .assign_advice(|| "res=lhs+rhs", config.advice[0], 1, || value)
                .map(Number)
        },
    )
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = [meta.advice_column(), meta.advice_column()];
        let instance = meta.instance_column();
        let constant = meta.fixed_column();

        meta.enable_equality(instance);
        meta.enable_constant(constant);

        for c in &advice {
            meta.enable_equality(*c);
        }
        let s_mul = meta.selector();
        let s_add = meta.selector();

        /* Gate design:
              | a0  |  a1 | s_mul |
              | ----|-----|-------|
              | lhs | rhs | s_mul |
              | out |     |       |
        */
        meta.create_gate("mul_gate", |meta| {
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[0], Rotation::next());
            let s_mul = meta.query_selector(s_mul);
            // vec![s_mul * (lhs * rhs - out)]
            Constraints::with_selector(s_mul, vec![(lhs * rhs - out)])
        });

        /* Gate design:
              | a0  |  a1 | s_add |
              | ----|-----|-------|
              | lhs | rhs | s_add |
              | out |     |       |
        */
        meta.create_gate("add_gate", |meta| {
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[0], Rotation::next());
            let s_add = meta.query_selector(s_add);
            // vec![s_add * (lhs + rhs - out)]
            Constraints::with_selector(s_add, vec![(lhs + rhs - out)])
        });

        CircuitConfig {
            advice,
            instance,
            s_mul,
            s_add,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let a = load_private(&config, layouter.namespace(|| "load a"), self.a)?;
        let b = load_private(&config, layouter.namespace(|| "load b"), self.b)?;
        let c = load_private(&config, layouter.namespace(|| "load c"), self.c)?;
        let x1 = load_private(&config, layouter.namespace(|| "load x1"), self.x1)?;
        let x2 = load_private(&config, layouter.namespace(|| "load x2"), self.x2)?;
        let x3 = load_private(&config, layouter.namespace(|| "load x3"), self.x3)?;

        let x1a = mul(&config, layouter.namespace(|| "x1*a"), x1, a)?;
        let x2b = mul(&config, layouter.namespace(|| "x2*b"), x2, b)?;
        let x3c = mul(&config, layouter.namespace(|| "x3*c"), x3, c)?;
        let t1 = add(&config, layouter.namespace(|| "x1a+x2b"), x1a, x2b)?;
        let out = add(&config, layouter.namespace(|| "t1+x3c"), t1, x3c)?;

        //expose public
        layouter
            .namespace(|| "expose out")
            .constrain_instance(out.0.cell(), config.instance, 0)
    }
}

// This is the main interface accessed by caller
pub fn generate_proof() -> (Vec<u8>, Vec<u8>) {
    use std::{
        fs::File,
        io::{BufReader, BufWriter, Write},
    };

    use halo2_proofs::{
        plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, ProvingKey},
        poly::kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverGWC, VerifierGWC},
            strategy::SingleStrategy,
        },
        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
        SerdeFormat,
    };
    use halo2curves::bn256::{Bn256, Fr, G1Affine};
    use rand_core::OsRng;

    // ANCHOR: test-circuit
    // The number of rows in our circuit cannot exceed 2^k. Since our example
    // circuit is very small, we can pick a very small value here.
    let k = 5;

    // Prepare the private and public inputs to the circuit!
    let a = Fr::from(1);
    let b = Fr::from(2);
    let c = Fr::from(3);
    let x1 = Fr::from(10);
    let x2 = Fr::from(12);
    let x3 = Fr::from(13);
    let out = x1 * a + x2 * b + x3 * c;
    println!("Public out=:{:?}", out);

    // Instantiate the circuit with the private inputs.
    let circuit = MyCircuit {
        a: Value::known(a),
        b: Value::known(b),
        c: Value::known(c),
        x1: Value::known(x1),
        x2: Value::known(x2),
        x3: Value::known(x3),
    };

    let params = ParamsKZG::<Bn256>::setup(k, OsRng);
    let vk = keygen_vk(&params, &circuit).expect("vk should not fail");
    let pk = keygen_pk(&params, vk, &circuit).expect("pk should not fail");

    let f = File::create("serialization-test.pk").unwrap();
    let mut writer = BufWriter::new(f);
    pk.write(&mut writer, SerdeFormat::RawBytes).unwrap();
    writer.flush().unwrap();

    let f = File::open("serialization-test.pk").unwrap();
    let mut reader = BufReader::new(f);
    #[allow(clippy::unit_arg)]
    let pk = ProvingKey::<G1Affine>::read::<_, MyCircuit<Fr>>(
        &mut reader,
        SerdeFormat::RawBytes,
        #[cfg(feature = "circuit-params")]
        circuit.params(),
    )
    .unwrap();
    // println!("The pk is: {:?}", pk);

    // std::fs::remove_file("serialization-test.pk").unwrap();

    let vu8_out = out.to_raw_bytes();

    let instances: &[&[Fr]] = &[&[out]];
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof::<
        KZGCommitmentScheme<Bn256>,
        ProverGWC<'_, Bn256>,
        Challenge255<G1Affine>,
        _,
        Blake2bWrite<Vec<u8>, G1Affine, Challenge255<_>>,
        _,
    >(
        &params,
        &pk,
        &[circuit],
        &[instances],
        OsRng,
        &mut transcript,
    )
    .expect("prover should not fail");
    let proof = transcript.finalize();
    println!("The proof is: {:?}", proof);

    /*
        let strategy = SingleStrategy::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        assert!(verify_proof::<
            KZGCommitmentScheme<Bn256>,
            VerifierGWC<'_, Bn256>,
            Challenge255<G1Affine>,
            Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
            SingleStrategy<'_, Bn256>,
        >(
            &params,
            pk.get_vk(),
            strategy,
            &[instances],
            &mut transcript
        )
        .is_ok());
    */

    (vu8_out, proof)
}
