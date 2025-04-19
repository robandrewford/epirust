#[cfg(test)]
mod tests {
    use epirust::datasets::{DatasetManager, DatasetConfig};
    use tempfile::TempDir;
    use std::path::PathBuf;
    use tokio;

    #[tokio::test]
    async fn test_dataset_download() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DatasetManager::new();
        
        let test_dataset = DatasetConfig {
            name: "framingham".to_string(),
            url: "https://raw.githubusercontent.com/paulhendricks/datasets/master/data-raw/framingham/framingham.csv".to_string(),
            citation: "Test citation".to_string(),
            description: "Test description".to_string(),
        };

        let result = manager.download_dataset(&test_dataset).await;
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.is_file());
    }

    #[test]
    fn test_test_data() {
        use epirust::datasets::test_data::create_test_dataset;
        let data = create_test_dataset();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0].len(), 3);
        assert_eq!(data[0][0], 1.0);
    }
} 