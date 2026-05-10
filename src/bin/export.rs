/// export.rs — AlphaRust Swing Export
/// Place at: src/bin/export.rs
///
/// Runs swing detection on all CSVs in src/test/test_data/
/// and writes one JSON file per CSV to src/test/test_data/output/json/
///
/// Run: cargo run --bin export --release

use AlphaRust::detectors::primatives::candles::candle::Candle;
use AlphaRust::detectors::swings::swing_streaming::{
    ConfirmationType, StremingSwingDetector, SwingMode, SwingType,
};
use AlphaRust::engine::ingester::CsvIngester;

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn clone_confirmation(c: &ConfirmationType) -> ConfirmationType {
    match c {
        ConfirmationType::PercentCandle(v) => ConfirmationType::PercentCandle(*v),
        ConfirmationType::PercentPrice(v)  => ConfirmationType::PercentPrice(*v),
        ConfirmationType::Ticks(t, s)      => ConfirmationType::Ticks(*t, *s),
        ConfirmationType::Candles(n)       => ConfirmationType::Candles(*n),
    }
}

struct ModeConfig {
    label:        &'static str,
    confirmation: fn() -> ConfirmationType,
}

fn mode_configs() -> Vec<ModeConfig> {
    vec![
        ModeConfig { label: "PercentCandle_0.3",  confirmation: || ConfirmationType::PercentCandle(0.003) },
        ModeConfig { label: "PercentPrice_0.1",   confirmation: || ConfirmationType::PercentPrice(0.001) },
        ModeConfig { label: "Ticks_4x0.25",       confirmation: || ConfirmationType::Ticks(4, 0.25) },
        ModeConfig { label: "Candles_2",           confirmation: || ConfirmationType::Candles(2) },
    ]
}

fn format_num(n: usize) -> String {
    let s = n.to_string();
    let mut r = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { r.push(','); }
        r.push(ch);
    }
    r.chars().rev().collect()
}

fn main() {
    let data_dir   = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test/test_data");
    let output_dir = data_dir.join("output").join("json");
    fs::create_dir_all(&output_dir).expect("Failed to create json output dir");

    let mut csv_files: Vec<PathBuf> = fs::read_dir(&data_dir)
        .expect("Failed to read test_data dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "csv").unwrap_or(false))
        .collect();
    csv_files.sort();

    if csv_files.is_empty() {
        eprintln!("No CSV files found in {:?}", data_dir);
        return;
    }

    println!("\n╔══ AlphaRust JSON Export ════════════════════════════════════╗");
    println!("║  Output: {:?}", output_dir);
    println!("╚════════════════════════════════════════════════════════════╝\n");

    for csv_path in &csv_files {
        let filename = csv_path.file_name().unwrap().to_string_lossy().to_string();
        println!("▶  {}", filename);

        // Ingest candles once
        let t0 = Instant::now();
        let file = File::open(csv_path).expect("Failed to open CSV");
        let candles: Vec<Candle> = CsvIngester::new(file).collect();
        let ingest_ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!("   ingested {} candles in {:.1}ms", format_num(candles.len()), ingest_ms);

        // Build candle JSON array
        let candle_json: String = candles.iter().map(|c| format!(
            "{{\"t\":{},\"o\":{:.2},\"h\":{:.2},\"l\":{:.2},\"c\":{:.2}}}",
            c.timestamp, c.open, c.high, c.low, c.close
        )).collect::<Vec<_>>().join(",");

        // Run each mode
        let modes = mode_configs();
        let mut mode_jsons: Vec<String> = Vec::new();

        for mode in &modes {
            let conf_high = (mode.confirmation)();
            let conf_low  = clone_confirmation(&conf_high);

            let t1 = Instant::now();

            let mut high_det = StremingSwingDetector::new(
                SwingMode::Wick, SwingType::High, conf_high, 1000,
            );
            for &c in &candles { high_det.process_candle(c); }
            let high_swings = high_det.get_swings();

            let mut low_det = StremingSwingDetector::new(
                SwingMode::Wick, SwingType::Low, conf_low, 1000,
            );
            for &c in &candles { low_det.process_candle(c); }
            let low_swings = low_det.get_swings();

            let detect_ms = t1.elapsed().as_secs_f64() * 1000.0;

            println!(
                "   {:25} ↑{} ↓{} swings in {:.1}ms",
                mode.label,
                format_num(high_swings.len()),
                format_num(low_swings.len()),
                detect_ms,
            );

            // Build swing JSON
            let swing_pts: String = high_swings.iter()
                .map(|(ts, _, _)| {
                    // find the candle to get the actual high price
                    let price = candles.iter()
                        .find(|c| c.timestamp == *ts)
                        .map(|c| c.high)
                        .unwrap_or(0.0);
                    format!("{{\"t\":{},\"p\":{:.2},\"type\":\"high\"}}", ts, price)
                })
                .chain(low_swings.iter().map(|(ts, _, _)| {
                    let price = candles.iter()
                        .find(|c| c.timestamp == *ts)
                        .map(|c| c.low)
                        .unwrap_or(0.0);
                    format!("{{\"t\":{},\"p\":{:.2},\"type\":\"low\"}}", ts, price)
                }))
                .collect::<Vec<_>>()
                .join(",");

            let candles_per_sec = candles.len() as f64 / ((ingest_ms + detect_ms) / 1000.0);

            mode_jsons.push(format!(
                "{{\"label\":{:?},\"ingest_ms\":{:.1},\"detect_ms\":{:.1},\"candles_per_sec\":{:.0},\"swings\":[{}]}}",
                mode.label, ingest_ms, detect_ms, candles_per_sec, swing_pts
            ));
        }

        // Write JSON file
        let json = format!(
            "{{\"file\":{:?},\"candle_count\":{},\"candles\":[{}],\"modes\":[{}]}}",
            filename,
            candles.len(),
            candle_json,
            mode_jsons.join(",")
        );

        let out_path = output_dir.join(format!("{}.json", filename));
        let mut f = File::create(&out_path).expect("Failed to create JSON");
        f.write_all(json.as_bytes()).expect("Failed to write JSON");
        println!("   ✓ → {:?}\n", out_path);
    }

    println!("Done. Run `python viz.py` to launch the dashboard.");
}