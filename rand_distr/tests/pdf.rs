use average::Histogram;
use rand::{Rng, SeedableRng};
use rand_distr::Normal;

average::define_histogram!(hist, 100);
use hist::Histogram as Histogram100;

mod sparkline;

#[test]
fn normal() {
    const N_SAMPLES: u64 = 1_000_000;
    const MEAN: f64 = 2.;
    const STD_DEV: f64 = 0.5;
    const MIN_X: f64 = -1.;
    const MAX_X: f64 = 5.;

    let dist = Normal::new(MEAN, STD_DEV).unwrap();
    let mut hist = Histogram100::with_const_width(MIN_X,MAX_X);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);

    for _ in 0..N_SAMPLES {
        let _ = hist.add(rng.sample(dist));  // Ignore out-of-range values
    }

    println!("Sampled normal distribution:\n{}",
        sparkline::render_u64_as_string(hist.bins()));

    fn pdf(x: f64) -> f64 {
        (-0.5 * ((x - MEAN) / STD_DEV).powi(2)).exp() /
            (STD_DEV * (2. * core::f64::consts::PI).sqrt())
    }

    let mut bin_centers = hist.centers();
    let mut expected = [0.; 100];
    for e in &mut expected {
        *e = pdf(bin_centers.next().unwrap());
    }

    println!("Expected normal distribution:\n{}",
        sparkline::render_u64_as_string(hist.bins()));

    let mut normalized_bins= hist.normalized_bins();
    let mut diff = [0.; 100];
    for i in 0..100 {
        let bin = (normalized_bins.next().unwrap() as f64) / (N_SAMPLES as f64) ;
        diff[i] = (bin - expected[i]).abs();
    }

    println!("Difference:\n{}",
        sparkline::render_f64_as_string(&diff[..]));
    println!("max diff: {:?}", diff.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));

    // Check that the differences are significantly smaller than the expected error.
    let mut expected_error = [0.; 100];
    // Calculate error from histogram
    for (err, var) in expected_error.iter_mut().zip(hist.variances()) {
        *err = var.sqrt() / (N_SAMPLES as f64);
    }
    // Normalize error by bin width
    for (err, width) in expected_error.iter_mut().zip(hist.widths()) {
        *err /= width;
    }
    // TODO: Calculate error from distribution cutoff / normalization

    println!("max expected_error: {:?}", expected_error.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
    for (&d, &e) in diff.iter().zip(expected_error.iter()) {
        // Difference larger than 3 standard deviations or cutoff
        let tol = (3. * e).max(1e-4);
        if d > tol {
            panic!("Difference = {} * tol", d / tol);
        }
    }
}