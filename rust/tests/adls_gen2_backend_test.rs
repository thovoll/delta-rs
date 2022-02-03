#[cfg(feature = "azure")]
mod adls_gen2_backend {
    use azure_storage::storage_shared_key_credential::StorageSharedKeyCredential;
    use azure_storage_datalake::clients::DataLakeClient;
    use chrono::Utc;
    use deltalake::StorageError;
    use serial_test::serial;
    use std::env;

    /*
     * An Azure Data Lake Gen2 Storage Account is required to run these tests and must be provided by
     * the developer. Because of this requirement, the tests cannot run in CI and are therefore marked
     * #[ignore]. As a result, the developer must execute these tests on their machine.
     * In order to execute tests, remove the desired #[ignore] below and execute via:
     * 'cargo test --features azure --test adls_gen2_backend_test -- --nocapture'
     * `AZURE_STORAGE_ACCOUNT_NAME` is required to be set in the environment.
     * `AZURE_STORAGE_ACCOUNT_KEY` is required to be set in the environment.
     */
    #[ignore]
    #[tokio::test]
    #[serial]
    async fn test_put_and_delete_obj_with_dir() {
        // Arrange
        let storage_account_name = env::var("AZURE_STORAGE_ACCOUNT_NAME").unwrap();
        let storage_account_key = env::var("AZURE_STORAGE_ACCOUNT_KEY").unwrap();

        let data_lake_client = DataLakeClient::new(
            StorageSharedKeyCredential::new(
                storage_account_name.to_owned(),
                storage_account_key.to_owned(),
            ),
            None,
        );

        // Create a new file system for test isolation
        let file_system_name = format!("test-adlsgen2-backend-{}", Utc::now().timestamp());
        let file_system_client =
            data_lake_client.into_file_system_client(file_system_name.to_owned());
        file_system_client.create().into_future().await.unwrap();

        let table_uri = &format!("adls2://{}/{}/", storage_account_name, file_system_name);
        let backend = deltalake::get_backend_for_uri(table_uri).unwrap();

        let ts = Utc::now().timestamp();
        let file_path = &format!("{}put-and-delete-obj-with-dir/file-{}.txt", table_uri, ts);

        // Act 1
        backend.put_obj(file_path, &[12, 13, 14]).await.unwrap();

        // Assert 1
        let file_meta_data = backend.head_obj(file_path).await.unwrap();
        assert_eq!(file_meta_data.path, *file_path);

        // Act 2
        // Note: dir1 itself does not get deleted here, just the file
        backend.delete_obj(file_path).await.unwrap();

        // Assert 2
        let head_err = backend.head_obj(file_path).await.err().unwrap();
        assert!(matches!(head_err, StorageError::NotFound));

        // Cleanup
        file_system_client.delete().into_future().await.unwrap();
    }
}
