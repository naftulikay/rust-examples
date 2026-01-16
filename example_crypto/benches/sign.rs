use criterion::measurement::Measurement;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion};
use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::sign::{Signer, Verifier};
use rand::prelude::SliceRandom;
use rand::{thread_rng, RngCore};
use std::fmt::{Display, Formatter};

const SIGN_COUNT: usize = 4096;

const SIGN_INDEX_REPEAT: usize = 4;

pub struct EndlessShuffledIter<'a, T> {
    data: &'a [T],
    indices: Vec<usize>,
    current: usize,
}

impl<'a, T> EndlessShuffledIter<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        Self::with_factor(data, SIGN_INDEX_REPEAT)
    }

    pub fn with_factor(data: &'a [T], factor: usize) -> Self {
        let mut indices = Vec::new();

        for _ in 0..factor {
            let mut i: Vec<usize> = (0..data.len()).collect();
            i.shuffle(&mut thread_rng());
            indices.extend(i);
        }

        Self {
            data,
            indices,
            current: 0,
        }
    }
}

impl<'a, T> Iterator for EndlessShuffledIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // get the current index by getting the index in the array
        let index = self.current;

        // update the index
        self.current = (self.current + 1) % self.data.len();

        // fetch the data
        self.data.get(index)
    }
}

pub struct SignBencherData<const S: usize> {
    pub items: Vec<SignBencher<S>>,
}

impl<const S: usize> SignBencherData<S> {
    pub fn new<F>(size: usize, factory: F, digest: Option<MessageDigest>) -> Self
    where
        F: Fn() -> PKey<Private>,
    {
        Self {
            items: (0..size)
                .map(|_| SignBencher::generate_valid(factory(), digest))
                .collect(),
        }
    }
}

pub struct SignBencher<const S: usize> {
    pub data: [u8; S],
    pub signature: Vec<u8>,
    pub key: PKey<Private>,
    pub digest: Option<MessageDigest>,
}

impl<const S: usize> SignBencher<S> {
    pub fn generate_valid(key: PKey<Private>, digest: Option<MessageDigest>) -> Self {
        let data = {
            let mut b = [0; S];
            thread_rng().fill_bytes(&mut b);
            b
        };

        let signature = match digest.as_ref() {
            Some(md) => Signer::new(*md, &key)
                .unwrap()
                .sign_oneshot_to_vec(&data)
                .unwrap(),
            None => Signer::new_without_digest(&key)
                .unwrap()
                .sign_oneshot_to_vec(&data)
                .unwrap(),
        };

        Self {
            data,
            signature,
            key,
            digest,
        }
    }

    pub fn verify(&self) -> bool {
        match &self.digest {
            Some(md) => Verifier::new(*md, &self.key)
                .unwrap()
                .verify(self.signature.as_slice())
                .unwrap(),
            None => Verifier::new_without_digest(&self.key)
                .unwrap()
                .verify(self.signature.as_slice())
                .unwrap(),
        }
    }
}

struct BenchConfig {
    algo: BenchAlgo,
    digest: BenchDigest,
}

impl BenchConfig {
    pub fn ecdsa(algo: BenchAlgo, digest: BenchDigest) -> Self {
        Self { algo, digest }
    }

    pub fn eddsa(algo: BenchAlgo) -> Self {
        Self {
            algo,
            digest: BenchDigest::Null,
        }
    }
}

impl Display for BenchConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.digest.eq(&BenchDigest::Null) {
            write!(f, "{}", self.algo)
        } else {
            write!(f, "{}::{}", self.algo, self.digest)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum BenchAlgo {
    Prime256V1,
    Secp256K1,
    Secp384R1,
    Ed448,
    Ed25519,
}

impl BenchAlgo {
    pub fn gen_key(&self) -> PKey<Private> {
        match &self {
            Self::Ed25519 => PKey::generate_ed25519().unwrap(),
            Self::Ed448 => PKey::generate_ed448().unwrap(),
            Self::Prime256V1 => PKey::from_ec_key(
                EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
            )
            .unwrap(),
            Self::Secp256K1 => PKey::from_ec_key(
                EcKey::generate(&EcGroup::from_curve_name(Nid::SECP256K1).unwrap()).unwrap(),
            )
            .unwrap(),
            Self::Secp384R1 => PKey::from_ec_key(
                EcKey::generate(&EcGroup::from_curve_name(Nid::SECP384R1).unwrap()).unwrap(),
            )
            .unwrap(),
        }
    }
}

impl Display for BenchAlgo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::Prime256V1 => "prime256v1",
                Self::Secp256K1 => "secp256k1",
                Self::Secp384R1 => "secp384r1",
                Self::Ed25519 => "ed25519",
                Self::Ed448 => "ed448",
            }
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
enum BenchDigest {
    Null,
    Sha256,
    #[allow(unused)]
    Sha384,
    #[allow(unused)]
    Sha512,
}

impl BenchDigest {
    pub fn to_md(&self) -> Option<MessageDigest> {
        match &self {
            Self::Null => None,
            Self::Sha256 => Some(MessageDigest::sha256()),
            Self::Sha384 => Some(MessageDigest::sha384()),
            Self::Sha512 => Some(MessageDigest::sha512()),
        }
    }
}

impl Display for BenchDigest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::Null => "none",
                Self::Sha256 => "sha256",
                Self::Sha384 => "sha384",
                Self::Sha512 => "sha512",
            }
        )
    }
}

fn bench(c: &mut Criterion) {
    bench_sized::<32>(c);
    bench_sized::<64>(c);
}

fn bench_sized<const S: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("openssl::verify::{S}"));

    bench_sized_individual::<S, _>(&mut group, BenchConfig::eddsa(BenchAlgo::Ed25519));
    bench_sized_individual::<S, _>(&mut group, BenchConfig::eddsa(BenchAlgo::Ed448));
    bench_sized_individual::<S, _>(
        &mut group,
        BenchConfig::ecdsa(BenchAlgo::Prime256V1, BenchDigest::Sha256),
    );
    bench_sized_individual::<S, _>(
        &mut group,
        BenchConfig::ecdsa(BenchAlgo::Secp256K1, BenchDigest::Sha256),
    );
    bench_sized_individual::<S, _>(
        &mut group,
        BenchConfig::ecdsa(BenchAlgo::Secp384R1, BenchDigest::Sha256),
    );
}

fn bench_sized_individual<const S: usize, M>(g: &mut BenchmarkGroup<M>, config: BenchConfig)
where
    M: Measurement,
{
    // prewarm
    let data =
        SignBencherData::<S>::new(SIGN_COUNT, || config.algo.gen_key(), config.digest.to_md());

    // create endless iterator
    let mut iter = EndlessShuffledIter::new(data.items.as_slice());

    g.bench_function(format!("{}", config), |b| {
        b.iter(|| black_box(iter.next().unwrap().verify()));
    });
}

criterion_group! {
    name = sign;
    config = Criterion::default();
    targets = bench,
}

criterion_main!(sign);
