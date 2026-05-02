use std::fs::File;
use std::io::{BufWriter, Write};

fn generate_candles(filename: &str, count: usize) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    // Write the header
    writeln!(writer, "timestamp,open,high,low,close,is_closed")?;

    for i in 0..count {
        // Create synthetic data
        let timestamp = 1625097600 + (i * 60); // Increments by 1 minute
        let open = 30000.0 + (i as f64 * 0.1);
        let high = open + 5.0;
        let low = open - 5.0;
        let close = open + 2.0;
        let is_closed = "True"; // Using the capitalized version to test your fix!

        writeln!(
            writer,
            "{},{},{},{},{},{}",
            timestamp, open, high, low, close, is_closed
        )?;
    }

    writer.flush()?; // Ensure everything is written to disk
    println!("Successfully generated {} lines in {}", count, filename);
    Ok(())
}

fn main() {
    // Generate both files
    generate_candles("/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/candles_100k.csv", 100_000).unwrap();
    generate_candles("/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/candles_500k.csv", 500_000).unwrap();
    generate_candles("/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/candles_1M.csv", 1_000_000).unwrap();
}
