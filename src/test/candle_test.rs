#[cfg(test)]
mod test {
    // 1. Move imports INSIDE the module room
    use crate::engine::ingester::CsvIngester;
    use std::time::Instant;
    use std::fs::File;

    #[test]
    fn candle_ingestion_test() -> Result<(), Box<dyn std::error::Error>> {
        let file_location = "/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/test_candles.csv";
        
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
    fn performance_test() -> Result<(), Box<dyn std::error::Error>>{
        let file_location = "/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/test_candles.csv";
        
        // 2. Use '?' to handle the Result of opening the file
        let test_data = File::open(file_location)?;

        // 3. Create the ingester
        let mut ingester = CsvIngester::new(test_data);


        let start = Instant::now();
        let mut count = 0;
        let start = std::time::Instant::now();
        let total = ingester.count(); // Runs the engine at full speed
        let duration = start.elapsed();

        let speed = total as f64 / duration.as_secs_f64();
        println!("--- PERFORMANCE REPORT ---");
        println!("Total Candles: {}", total);
        println!("Total Time: {:?}", duration);
        println!("Speed: {:.2} candles/sec", speed);

        Ok(())
    }

    #[test]
    fn performance_test_100K() -> Result<(), Box<dyn std::error::Error>>{
        let file_location = "/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/candles_100k.csv";
        
        // 2. Use '?' to handle the Result of opening the file
        let test_data = File::open(file_location)?;

        // 3. Create the ingester
        let mut ingester = CsvIngester::new(test_data);


        let start = Instant::now();
        let mut count = 0;
        let start = std::time::Instant::now();
        let total = ingester.count(); // Runs the engine at full speed
        let duration = start.elapsed();

        let speed = total as f64 / duration.as_secs_f64();
        println!("--- PERFORMANCE REPORT ---");
        println!("Total Candles: {}", total);
        println!("Total Time: {:?}", duration);
        println!("Speed: {:.2} candles/sec", speed);

        Ok(())
    }

    #[test]
    fn performance_test_500K() -> Result<(), Box<dyn std::error::Error>>{
        let file_location = "/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/candles_500k.csv";
        
        // 2. Use '?' to handle the Result of opening the file
        let test_data = File::open(file_location)?;

        // 3. Create the ingester
        let mut ingester = CsvIngester::new(test_data);


        let start = Instant::now();
        let mut count = 0;
        let start = std::time::Instant::now();
        let total = ingester.count(); // Runs the engine at full speed
        let duration = start.elapsed();

        let speed = total as f64 / duration.as_secs_f64();
        println!("--- PERFORMANCE REPORT ---");
        println!("Total Candles: {}", total);
        println!("Total Time: {:?}", duration);
        println!("Speed: {:.2} candles/sec", speed);

        Ok(())
    }

     #[test]
    fn performance_test_1M() -> Result<(), Box<dyn std::error::Error>>{
        let file_location = "/Users/dakotahtorres/Desktop/Organized_Desktop/AlphaRust/src/test/test_data/candles_1M.csv";
        
        // 2. Use '?' to handle the Result of opening the file
        let test_data = File::open(file_location)?;

        // 3. Create the ingester
        let mut ingester = CsvIngester::new(test_data);


        let start = Instant::now();
        let mut count = 0;
        let start = std::time::Instant::now();
        let total = ingester.count(); // Runs the engine at full speed
        let duration = start.elapsed();

        let speed = total as f64 / duration.as_secs_f64();
        println!("--- PERFORMANCE REPORT ---");
        println!("Total Candles: {}", total);
        println!("Total Time: {:?}", duration);
        println!("Speed: {:.2} candles/sec", speed);

        Ok(())
    }
}