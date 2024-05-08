/// Prove knowing knowledge of six private inputs: x1 x2 x3 a b c
/// s.t: x1a + x2b + x3c = out
use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{
        create_proof, keygen_pk, keygen_vk, Advice, Circuit, Column, ConstraintSystem, Constraints,
        Error, Instance, Selector,
    },
    poly::{commitment::Params, Rotation},
};

use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::transcript::{Blake2bWrite, Challenge255};

// use halo2curves::bn256::{Bn256, Fr, G1Affine};
use rand_core::OsRng;
use std::fmt::Debug;

pub mod traits;
pub use traits::ZkTraitHalo2;

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

#[derive(Default, Clone)]
struct MyCircuit<F: Field> {
    coefs: Vec<Value<F>>,
    xs: Vec<Value<F>>,
}

fn load_private<F: Field>(
    config: &CircuitConfig,
    mut layouter: impl Layouter<F>,
    coef_value: Value<F>,
    x_value: Value<F>,
) -> Result<(Number<F>, Number<F>), Error> {
    layouter.assign_region(
        || "load private",
        |mut region| {
            let coef = region
                .assign_advice(|| "private input coef", config.advice[0], 0, || coef_value)
                .map(Number);

            let x = region
                .assign_advice(|| "private input x", config.advice[1], 0, || x_value)
                .map(Number);

            Ok((coef?, x?))
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
        let len = self.coefs.len();
        let mut number_vec: Vec<(Number<F>, Number<F>)> = Vec::new();
        for i in 0..len {
            let a_pair = load_private(
                &config,
                layouter.namespace(|| "load private pair of coef and x"),
                self.coefs[i].clone(),
                self.xs[i].clone(),
            )?;
            number_vec.push(a_pair);
        }

        let mut product_vec: Vec<Number<F>> = Vec::new();
        for i in 0..len {
            let product = mul(
                &config,
                layouter.namespace(|| "coef * x"),
                number_vec[i].0.clone(),
                number_vec[i].1.clone(),
            )?;
            product_vec.push(product);
        }

        let mut vleft = product_vec[0].clone();
        for i in 1..product_vec.len() {
            let vright = product_vec[i].clone();

            vleft = add(
                &config,
                layouter.namespace(|| "vleft + vright"),
                vleft,
                vright,
            )?;
        }

        let out = vleft;

        //expose public
        layouter
            .namespace(|| "expose out")
            .constrain_instance(out.0.cell(), config.instance, 0)
    }
}

pub fn gen_proof(coefs: Vec<Fp>, xs: Vec<Fp>) -> Result<(Vec<u8>, Vec<u8>), traits::Error> {
    // ANCHOR: test-circuit
    // The number of rows in our circuit cannot exceed 2^k. Since our example
    // circuit is very small, we can pick a very small value here.
    let k = 7;

    // Prepare the private and public inputs to the circuit!
    use std::iter::zip;

    let sum: Fp = zip(coefs.clone(), xs.clone())
        .map(|(coef, x)| coef * x)
        .sum();
    let out = sum;
    // println!("Public out=:{:?}", out);
    let pubinputs = vec![out];

    let coefs = coefs.into_iter().map(Value::known).collect();
    let xs = xs.into_iter().map(Value::known).collect();

    // Instantiate the circuit with the private inputs.
    let circuit = MyCircuit { coefs, xs };

    let params: Params<EqAffine> = Params::new(k);
    let vk = keygen_vk(&params, &circuit).expect("vk should not fail");
    let pk = keygen_pk(&params, vk, &circuit).expect("pk should not fail");

    let instances: &[&[Fp]] = &[&pubinputs];
    // let vu8_out = out.to_raw_bytes();
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    // Create a proof
    create_proof(
        &params,
        &pk,
        &[circuit.clone()],
        &[instances],
        OsRng,
        &mut transcript,
    )
    .expect("proof generation should not fail");
    let proof: Vec<u8> = transcript.finalize();

    // std::fs::write("./tests/plonk_api_proof.bin", &proof[..])
    // .expect("should succeed to write new proof");

    // Check that a hardcoded proof is satisfied
    // let proof = std::fs::read("./tests/plonk_api_proof.bin").expect("should succeed to read proof");
    // let strategy = SingleVerifier::new(&params);
    // let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    // assert!(verify_proof(
    //     &params,
    //     pk.get_vk(),
    //     strategy,
    //     &[&[&pubinputs[..]], &[&pubinputs[..]]],
    //     &mut transcript,
    // )
    // .is_ok());

    let vecu8_out = format!("{:?}", out).as_bytes().to_vec();
    Ok((vecu8_out, proof))
}
#[derive(Debug)]
pub struct ZKT;

impl traits::ZkTraitHalo2 for ZKT {
    type F = Fp;
    fn gen_proof(
        &self,
        coefs: Vec<Self::F>,
        xs: Vec<Self::F>,
        // TODO: add other parameters
        // e.g. setup parameters
    ) -> Result<(Vec<u8>, Vec<u8>), traits::Error> {
        gen_proof(coefs, xs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::zip;

    use halo2_proofs::circuit::Value;

    use halo2_proofs::pasta::{EqAffine, Fp};
    use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier};

    use halo2_proofs::poly::commitment::Params;
    use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};

    // use halo2curves::bn256::{Bn256, Fr, G1Affine};
    use rand_core::OsRng;

    #[test]
    fn test_fp_proof() {
        // ANCHOR: test-circuit
        // The number of rows in our circuit cannot exceed 2^k. Since our example
        // circuit is very small, we can pick a very small value here.
        let k = 13;
        const LEN: u64 = 1000;

        // Prepare the private and public inputs to the circuit!
        let coefs: Vec<_> = (1..LEN).map(Fp::from).collect();
        let xs: Vec<_> = (1..LEN).map(Fp::from).collect();
        // let x1 = Fp::from(10);
        // let x2 = Fp::from(12);
        // let x3 = Fp::from(13);
        // let coefs = vec![x1, x2, x3];
        // let a = Fp::from(1);
        // let b = Fp::from(2);
        // let c = Fp::from(3);
        // let xs = vec![a, b, c];

        let sum: Fp = zip(coefs.clone(), xs.clone())
            .map(|(coef, x)| coef * x)
            .sum();
        let out = sum;
        println!("Public out=:{:?}", out);
        let pubinputs = vec![out];

        let coefs = coefs.into_iter().map(Value::known).collect();
        let xs = xs.into_iter().map(Value::known).collect();

        // Instantiate the circuit with the private inputs.
        let circuit = MyCircuit { coefs, xs };

        let params: Params<EqAffine> = Params::new(k);
        let vk = keygen_vk(&params, &circuit).expect("vk should not fail");
        let pk = keygen_pk(&params, vk, &circuit).expect("pk should not fail");

        let instances: &[&[Fp]] = &[&pubinputs];
        // let vu8_out = out.to_raw_bytes();
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        // Create a proof
        create_proof(
            &params,
            &pk,
            &[circuit.clone()],
            &[instances],
            OsRng,
            &mut transcript,
        )
        .expect("proof generation should not fail");
        let proof: Vec<u8> = transcript.finalize();

        // std::fs::write("./tests/plonk_api_proof.bin", &proof[..])
        //     .expect("should succeed to write new proof");

        // // Check that a hardcoded proof is satisfied
        // let proof =
        //     std::fs::read("./tests/plonk_api_proof.bin").expect("should succeed to read proof");
        let strategy = SingleVerifier::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        assert!(verify_proof(
            &params,
            pk.get_vk(),
            strategy,
            &[&[&pubinputs[..]]],
            &mut transcript,
        )
        .is_ok());
    }
}
