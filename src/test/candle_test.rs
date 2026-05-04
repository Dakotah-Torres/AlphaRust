#[cfg(test)]
mod test {
    // 1. Move imports INSIDE the module room
    use crate::engine::ingester::CsvIngester;
    use std::time::Instant;
    use std::fs::File;

    #[test]
    fn candle_ingestion_test() -> Result<(), Box<dyn std::error::Error>> {
        let file_location = "/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/candles_10k.csv";
        
        // 2. Use '?' to handle the Result of opening the file
        let test_data = File::open(file_location)?;

        // 3. Create the ingester
        let mut ingester = CsvIngester::new(test_data);

        // 4. Assert on a boolean (is_some check)
        assert!(ingester.next().is_some());

        // 5. Return Ok(()) to signal the test passed
        Ok(())
    }


    #[test]
    fn performance_test_all() -> Result<(), Box<dyn std::error::Error>> {
        let files = vec![
            ("10K", "candles_10k.csv"),
            ("100k", "candles_100k.csv"),
            ("500k", "candles_500k.csv"),
            ("1M", "candles_1M.csv"),
            ("10M", "candles_10M.csv")
        ];

        for (label, filename) in files {
            let path = format!(
                "{}/src/test/test_data/{}",
                env!("CARGO_MANIFEST_DIR"),
                filename
            );

            let test_data = File::open(&path)?;
            let ingester = CsvIngester::new(test_data);

            let start = std::time::Instant::now();
            let total = ingester.count();
            let duration = start.elapsed();
            let speed = total as f64 / duration.as_secs_f64();

            println!("--- {} ---", label);
            println!("Total Candles: {}", total);
            println!("Total Time: {:?}", duration);
            println!("Speed: {:.2} candles/sec", speed);
        }

        Ok(())
    }
}