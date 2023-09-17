use rand::SeedableRng;
use rand_pcg::Pcg64;
use sampling::{*, examples::coin_flips::*};
use statrs::distribution::{Binomial, Discrete};
use std::f64::consts::LOG10_E;
use std::sync::{RwLock, Arc};

type Wlc = Arc<RwLock<WangLandau1T<HistogramFast<usize>, rand_pcg::Lcg128Xsl64, CoinFlipSequence<rand_pcg::Lcg128Xsl64>, CoinFlipMove, (), usize>>>;
type Ent = EntropicSampling<HistogramFast<usize>, rand_pcg::Lcg128Xsl64, CoinFlipSequence<rand_pcg::Lcg128Xsl64>, CoinFlipMove, (), usize>;

pub struct CoinSeq{
    pub wl: Wlc,
    pub log_prob_true: Vec<f64>,
    pub entr: Ent
}

pub fn generate_cs(n: usize, seed: u64, step_size: usize, threshold: f64) -> CoinSeq
{
    let hist = HistUsizeFast::new_inclusive(0, n).unwrap();
    let mut rng = Pcg64::seed_from_u64(seed);
    let ensemble = CoinFlipSequence::new(
        n,
        Pcg64::from_rng(&mut rng).unwrap()
    );

    let mut wl: WangLandau1T<HistogramFast<usize>, rand_pcg::Lcg128Xsl64, CoinFlipSequence<rand_pcg::Lcg128Xsl64>, CoinFlipMove, (), usize> = WangLandau1T::new(
        threshold, // arbitrary threshold for `log_f`(see paper), 
                 // you have to try what is good for your model
        ensemble,
        Pcg64::from_rng(&mut rng).unwrap(),
        step_size,  // stepsize 1 is sufficient for this problem
        hist,
        100 // every 100 steps: check if WL can refine factor f
    ).unwrap();

    wl.init_greedy_heuristic(
        |coin_seq| Some(coin_seq.head_count()),
        Some(10_000) // if no valid state is found after 10_000 
                     // this returns an Err. If you do not want a step limit,
                     // you can use None here
    ).expect("Unable to find valid state within 10_000 steps!");

    let ent = EntropicSampling::from_wl(wl.clone()).unwrap();

    let binomial = Binomial::new(0.5, n as u64).unwrap();
    let log_prob_true: Vec<_> = (0..=n)
        .map(|k| LOG10_E*binomial.ln_pmf(k as u64))
        .collect();

    CoinSeq { wl: Arc::new(RwLock::new(wl)), log_prob_true, entr: ent }
}