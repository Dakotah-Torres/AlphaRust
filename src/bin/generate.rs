/// generate.rs — AlphaRust Test Data Generator
/// Place at: src/bin/generate.rs
/// DELETE src/bin/generator.rs after adding this file.
///
/// Usage:
///   cargo run --bin generate --release -- 100 1K 10K 100K
///   cargo run --bin generate --release -- 10K 100K 1M 10M

use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

// ── RNG ───────────────────────────────────────────────────────────────────────

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*state >> 33) as f64) / (u32::MAX as f64)
}

fn randn(state: &mut u64) -> f64 {
    let u1 = lcg(state).max(1e-10);
    let u2 = lcg(state);
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

// ── Candle generator ──────────────────────────────────────────────────────────

struct CandleGen {
    price:    f64,
    momentum: f64,
    rng:      u64,
}

impl CandleGen {
    fn new(start_price: f64, seed: u64) -> Self {
        Self { price: start_price, momentum: 0.0, rng: seed }
    }

    fn next(&mut self) -> (f64, f64, f64, f64) {
        self.momentum = self.momentum * 0.85 + randn(&mut self.rng) * 0.4;
        let body  = (randn(&mut self.rng).abs() * 10.0).max(0.5);
        let dir   = if self.momentum >= 0.0 { 1.0 } else { -1.0 };
        let open  = self.price;
        let close = open + dir * body + randn(&mut self.rng) * 2.5;
        let high  = open.max(close) + (randn(&mut self.rng).abs() * 6.0).max(0.2);
        let low   = open.min(close) - (randn(&mut self.rng).abs() * 6.0).max(0.2);
        self.price = (close + randn(&mut self.rng) * 1.5).max(1.0);
        (open, high, low, close)
    }
}

// ── Argument parsing ──────────────────────────────────────────────────────────

fn parse_size(arg: &str) -> Result<usize, String> {
    let u = arg.to_uppercase();
    if let Some(n) = u.strip_suffix('M') {
        Ok((n.parse::<f64>().map_err(|_| format!("bad input: {}", arg))? * 1_000_000.0) as usize)
    } else if let Some(n) = u.strip_suffix('K') {
        Ok((n.parse::<f64>().map_err(|_| format!("bad input: {}", arg))? * 1_000.0) as usize)
    } else {
        arg.parse::<usize>().map_err(|_| format!("bad input: {}", arg))
    }
}

fn label(n: usize) -> String {
    if n >= 1_000_000 && n % 1_000_000 == 0 { format!("{}M", n / 1_000_000) }
    else if n >= 1_000 && n % 1_000 == 0    { format!("{}K", n / 1_000) }
    else                                     { format!("{}", n) }
}

fn commas(n: usize) -> String {
    let s = n.to_string();
    let mut r = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { r.push(','); }
        r.push(ch);
    }
    r.chars().rev().collect()
}

// ── Write one file ────────────────────────────────────────────────────────────

fn write_csv(path: &PathBuf, count: usize, seed: u64) -> std::io::Result<()> {
    let mut w   = BufWriter::new(File::create(path)?);
    let mut cgen = CandleGen::new(45_000.0, seed);
    writeln!(w, "timestamp,open,high,low,close,is_closed")?;
    for i in 0..count {
        let ts = 1_700_000_000u64 + i as u64 * 60;
        let (o, h, l, c) = cgen.next();
        writeln!(w, "{},{:.2},{:.2},{:.2},{:.2},True", ts, o, h, l, c)?;
    }
    w.flush()
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: cargo run --bin generate --release -- <sizes>");
        eprintln!("  e.g. -- 100 1K 10K 100K");
        eprintln!("  e.g. -- 10K 100K 1M 10M");
        std::process::exit(1);
    }

    let sizes: Vec<usize> = args.iter().map(|a| {
        parse_size(a).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); })
    }).collect();

    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test/test_data");
    fs::create_dir_all(&out).expect("could not create src/test/test_data");

    println!("\n╔══ AlphaRust Generator ══════════════════════════════════════╗");
    println!("║  writing to: {:?}", out);
    println!("╚════════════════════════════════════════════════════════════╝\n");

    for (i, &count) in sizes.iter().enumerate() {
        let name = format!("candles_{}.csv", label(count));
        let path = out.join(&name);
        let seed = 0xDEAD_BEEF_u64.wrapping_add(i as u64 * 99991);

        print!("  {:>12} candles → {} ... ", commas(count), name);
        std::io::stdout().flush().ok();

        let t = Instant::now();
        write_csv(&path, count, seed).unwrap_or_else(|e| {
            eprintln!("failed: {}", e);
            std::process::exit(1);
        });
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        println!("done ({:.0}ms, {:.1}M rows/s)", ms, count as f64 / ms / 1000.0);
    }

    println!("\n✓ done — run `cargo run --bin swing_bench --release` to benchmark\n");
}