use std::fs::File;
use std::io::{BufWriter, Write};

// ── Config ──────────────────────────────────────────────────────────────────
// Add or remove entries here to generate more/fewer files.
// Format: (filename, candle_count)
const FILES: &[(&str, usize)] = &[
    ("swing_test_100.csv",   100),
    ("swing_test_1k.csv",  1_000),
    ("swing_test_10k.csv", 10_000),
];

const OUTPUT_DIR: &str = "src/test/test_data";

// ── Realistic candle generation ──────────────────────────────────────────────
// Uses a simple random walk with momentum + mean reversion so the output
// looks like real price action (trending legs, pullbacks, swings).

fn lcg(state: &mut u64) -> f64 {
    // A fast, seedable pseudo-random number generator (no external crates).
    // Returns a value in [0.0, 1.0).
    *state = state.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let shifted = (*state >> 33) as f64;
    shifted / (u32::MAX as f64)
}

fn randn(state: &mut u64) -> f64 {
    // Box-Muller: converts two uniform samples into a standard normal.
    let u1 = lcg(state).max(1e-10);
    let u2 = lcg(state);
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

struct CandleGen {
    price:    f64,   // current mid price
    momentum: f64,   // trending bias (mean-reverts toward 0)
    rng:      u64,   // RNG state
}

impl CandleGen {
    fn new(start_price: f64, seed: u64) -> Self {
        Self { price: start_price, momentum: 0.0, rng: seed }
    }

    /// Returns (open, high, low, close) for the next candle.
    fn next_candle(&mut self) -> (f64, f64, f64, f64) {
        // Momentum drifts randomly and mean-reverts
        self.momentum = self.momentum * 0.85 + randn(&mut self.rng) * 0.3;

        let body_size  = (randn(&mut self.rng).abs() * 8.0).max(0.5);
        let direction  = if self.momentum >= 0.0 { 1.0 } else { -1.0 };

        let open  = self.price;
        let close = open + direction * body_size + randn(&mut self.rng) * 2.0;

        // Wicks extend beyond the body
        let wick_top    = (randn(&mut self.rng).abs() * 5.0).max(0.2);
        let wick_bottom = (randn(&mut self.rng).abs() * 5.0).max(0.2);

        let high = open.max(close) + wick_top;
        let low  = open.min(close) - wick_bottom;

        // Next candle opens near this close (small gap sometimes)
        self.price = close + randn(&mut self.rng) * 1.5;
        self.price = self.price.max(1.0); // price can't go negative

        (open, high, low, close)
    }
}

// ── File writer ──────────────────────────────────────────────────────────────

fn generate_file(path: &str, count: usize, seed: u64) -> std::io::Result<()> {
    let file   = File::create(path)?;
    let mut w  = BufWriter::new(file);
    let mut generator = CandleGen::new(45_000.0, seed);

    writeln!(w, "timestamp,open,high,low,close,is_closed")?;

    for i in 0..count {
        let timestamp = 1_700_000_000u64 + (i as u64 * 60); // 1-min bars
        let (open, high, low, close) = generator.next_candle();

        writeln!(
            w,
            "{},{:.2},{:.2},{:.2},{:.2},True",
            timestamp, open, high, low, close
        )?;
    }

    w.flush()?;
    println!("✓  {} ({} candles)", path, count);
    Ok(())
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    println!("Generating swing test data...\n");

    for (i, (filename, count)) in FILES.iter().enumerate() {
        let path = format!("{}/{}", OUTPUT_DIR, filename);
        let seed = 0xDEAD_BEEF_u64.wrapping_add(i as u64 * 12345);
        generate_file(&path, *count, seed).unwrap();
    }

    println!("\nDone. Add entries to FILES[] in swing_generator.rs to generate more.");
}