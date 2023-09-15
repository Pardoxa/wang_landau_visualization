use rand::SeedableRng;
use rand_pcg::Pcg64;
use sampling::{*, examples::coin_flips::*};
use statrs::distribution::{Binomial, Discrete};
use std::f64::consts::LOG10_E;


pub struct CoinSeq{
    pub wl: WangLandau1T<HistogramFast<usize>, rand_pcg::Lcg128Xsl64, CoinFlipSequence<rand_pcg::Lcg128Xsl64>, CoinFlipMove, (), usize>,
    pub log_prob_true: Vec<f64>
}

pub fn generate_cs(n: usize, seed: u64, step_size: usize) -> CoinSeq
{
    let hist = HistUsizeFast::new_inclusive(0, n).unwrap();
    let mut rng = Pcg64::seed_from_u64(seed);
    let ensemble = CoinFlipSequence::new(
        n,
        Pcg64::from_rng(&mut rng).unwrap()
    );

    let mut wl: WangLandau1T<HistogramFast<usize>, rand_pcg::Lcg128Xsl64, CoinFlipSequence<rand_pcg::Lcg128Xsl64>, CoinFlipMove, (), usize> = WangLandau1T::new(
        0.000001, // arbitrary threshold for `log_f`(see paper), 
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
    let binomial = Binomial::new(0.5, n as u64).unwrap();
    let log_prob_true: Vec<_> = (0..=n)
        .map(|k| LOG10_E*binomial.ln_pmf(k as u64))
        .collect();

    CoinSeq { wl, log_prob_true }
}