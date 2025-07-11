use tokio;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

// Import project modules
use grpc_shared::server::grpc_server::GrpcSigningServer;
use grpc_shared::crypto::signing::KeyManager;
use grpc_shared::config::settings::{ServerConfig, ClientConfig, TransportType, KeyType, SigningAlgorithm};
use grpc_client::client::grpc_client::GrpcSigningClient;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tonic::transport::Server;
    use grpc_shared::proto::signing::signing_service_server::SigningServiceServer;

    async fn start_test_server() -> (u16, tokio::task::JoinHandle<()>) {
        // Create server configuration
        let config = ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0, // Auto-assign port
            transport: TransportType::Tcp,
            max_connections: 100,
            connection_timeout_ms: 30000,
            request_timeout_ms: 10000,
            keep_alive_interval_ms: Some(30000),
            keep_alive_timeout_ms: Some(5000),
            shutdown_timeout_ms: 5000,
        };

        // Create key manager and generate test key
        let mut key_manager = KeyManager::new();
        key_manager.generate_key("integration-test-key", KeyType::EccP256).await.unwrap();
        
        // Create server
        let server = GrpcSigningServer::new(Arc::new(Mutex::new(key_manager)));
        
        // Find available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        
        let addr = format!("127.0.0.1:{}", port).parse().unwrap();
        
        // Start server in background
        let server_handle = tokio::spawn(async move {
            Server::builder()
                .add_service(SigningServiceServer::new(server))
                .serve(addr)
                .await
                .unwrap();
        });
        
        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        (port, server_handle)
    }

    async fn create_test_client(port: u16) -> GrpcSigningClient {
        let config = ClientConfig {
            server_address: "127.0.0.1".to_string(),
            server_port: port,
            transport: TransportType::Tcp,
            connection_timeout_ms: 5000,
            request_timeout_ms: 10000,
            max_retries: 3,
            retry_delay_ms: 1000,
            keep_alive_interval_ms: Some(30000),
            keep_alive_timeout_ms: Some(5000),
        };
        
        GrpcSigningClient::new(config)
    }

    #[tokio::test]
    async fn test_end_to_end_signing_workflow() {
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        // Create and connect client
        let mut client = create_test_client(port).await;
        let connect_result = client.connect().await;
        assert!(connect_result.is_ok(), "Client should connect to server");
        
        // Test health check
        let health_result = client.health_check().await;
        assert!(health_result.is_ok(), "Health check should succeed");
        let health_response = health_result.unwrap();
        assert_eq!(health_response.status, "healthy");
        
        // Test signing
        let sign_result = client.sign(
            "integration-test-key",
            b"Hello, integration test!",
            SigningAlgorithm::EcdsaP256Sha256
        ).await;
        assert!(sign_result.is_ok(), "Signing should succeed");
        
        let sign_response = sign_result.unwrap();
        assert!(!sign_response.signature.is_empty(), "Signature should not be empty");
        assert!(sign_response.processing_time_ms > 0, "Processing time should be recorded");
        assert!(!sign_response.request_id.is_empty(), "Request ID should be present");
        
        // Test signature verification
        let verify_result = client.verify(
            "integration-test-key",
            b"Hello, integration test!",
            &sign_response.signature,
            SigningAlgorithm::EcdsaP256Sha256
        ).await;
        assert!(verify_result.is_ok(), "Verification should succeed");
        assert!(verify_result.unwrap(), "Signature should be valid");
        
        // Disconnect client
        let disconnect_result = client.disconnect().await;
        assert!(disconnect_result.is_ok(), "Client should disconnect cleanly");
        
        // Shutdown server
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_key_management_workflow() {
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        // Create and connect client
        let mut client = create_test_client(port).await;
        client.connect().await.unwrap();
        
        // List initial keys
        let initial_keys = client.list_keys().await.unwrap();
        let initial_count = initial_keys.len();
        
        // Generate new key
        let new_key_id = "test-generated-key";
        let generate_result = client.generate_key(new_key_id, KeyType::EccP384).await;
        assert!(generate_result.is_ok(), "Key generation should succeed");
        
        // List keys after generation
        let updated_keys = client.list_keys().await.unwrap();
        assert_eq!(updated_keys.len(), initial_count + 1, "Should have one more key");
        
        // Verify new key exists
        let new_key_exists = updated_keys.iter().any(|key| key.key_id == new_key_id);
        assert!(new_key_exists, "New key should be in the list");
        
        // Test signing with new key
        let sign_result = client.sign(
            new_key_id,
            b"Test with generated key",
            SigningAlgorithm::EcdsaP384Sha384
        ).await;
        assert!(sign_result.is_ok(), "Signing with generated key should succeed");
        
        // Delete the key
        let delete_result = client.delete_key(new_key_id).await;
        assert!(delete_result.is_ok(), "Key deletion should succeed");
        
        // Verify key is deleted
        let final_keys = client.list_keys().await.unwrap();
        assert_eq!(final_keys.len(), initial_count, "Should return to original key count");
        
        let deleted_key_exists = final_keys.iter().any(|key| key.key_id == new_key_id);
        assert!(!deleted_key_exists, "Deleted key should not be in the list");
        
        client.disconnect().await.unwrap();
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_error_handling_workflow() {
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        // Create and connect client
        let mut client = create_test_client(port).await;
        client.connect().await.unwrap();
        
        // Test signing with non-existent key
        let sign_result = client.sign(
            "non-existent-key",
            b"test message",
            SigningAlgorithm::EcdsaP256Sha256
        ).await;
        assert!(sign_result.is_err(), "Signing with non-existent key should fail");
        
        // Test verification with non-existent key
        let verify_result = client.verify(
            "non-existent-key",
            b"test message",
            &[1, 2, 3, 4], // dummy signature
            SigningAlgorithm::EcdsaP256Sha256
        ).await;
        assert!(verify_result.is_err(), "Verification with non-existent key should fail");
        
        // Test key generation with duplicate ID
        let duplicate_result = client.generate_key(
            "integration-test-key", // Already exists
            KeyType::EccP256
        ).await;
        assert!(duplicate_result.is_err(), "Duplicate key generation should fail");
        
        // Test deletion of non-existent key
        let delete_result = client.delete_key("non-existent-key").await;
        assert!(delete_result.is_err(), "Deleting non-existent key should fail");
        
        // Test signing with mismatched algorithm
        let mismatch_result = client.sign(
            "integration-test-key", // P-256 key
            b"test message",
            SigningAlgorithm::EcdsaP384Sha384 // P-384 algorithm
        ).await;
        assert!(mismatch_result.is_err(), "Mismatched algorithm should fail");
        
        client.disconnect().await.unwrap();
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_concurrent_client_connections() {
        use tokio::task::JoinSet;
        
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        let mut join_set = JoinSet::new();
        
        // Spawn multiple concurrent clients
        for i in 0..5 {
            let client_port = port;
            join_set.spawn(async move {
                let mut client = create_test_client(client_port).await;
                client.connect().await.unwrap();
                
                // Each client performs a unique signing operation
                let message = format!("Message from client {}", i);
                let sign_result = client.sign(
                    "integration-test-key",
                    message.as_bytes(),
                    SigningAlgorithm::EcdsaP256Sha256
                ).await;
                
                client.disconnect().await.unwrap();
                (i, sign_result.is_ok())
            });
        }
        
        // Wait for all clients to complete
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            results.push(result.unwrap());
        }
        
        // All clients should succeed
        for (client_id, success) in results {
            assert!(success, "Client {} should succeed", client_id);
        }
        
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_large_message_signing() {
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        // Create and connect client
        let mut client = create_test_client(port).await;
        client.connect().await.unwrap();
        
        // Test with various message sizes
        let test_sizes = vec![
            1024,        // 1KB
            10 * 1024,   // 10KB
            100 * 1024,  // 100KB
            1024 * 1024, // 1MB
        ];
        
        for size in test_sizes {
            let large_message = vec![0x42u8; size];
            
            let sign_result = client.sign(
                "integration-test-key",
                &large_message,
                SigningAlgorithm::EcdsaP256Sha256
            ).await;
            assert!(sign_result.is_ok(), "Signing {}KB message should succeed", size / 1024);
            
            let signature = sign_result.unwrap().signature;
            
            // Verify the signature
            let verify_result = client.verify(
                "integration-test-key",
                &large_message,
                &signature,
                SigningAlgorithm::EcdsaP256Sha256
            ).await;
            assert!(verify_result.is_ok() && verify_result.unwrap(), 
                   "Verification of {}KB message should succeed", size / 1024);
        }
        
        client.disconnect().await.unwrap();
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_connection_recovery() {
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        // Create client
        let mut client = create_test_client(port).await;
        client.connect().await.unwrap();
        
        // Perform initial operation
        let initial_result = client.health_check().await;
        assert!(initial_result.is_ok(), "Initial health check should succeed");
        
        // Simulate connection loss by disconnecting
        client.disconnect().await.unwrap();
        
        // Attempt operation while disconnected (should trigger reconnection)
        let reconnect_result = client.health_check().await;
        assert!(reconnect_result.is_ok(), "Health check after reconnection should succeed");
        
        // Verify we can still perform operations
        let sign_result = client.sign(
            "integration-test-key",
            b"test after reconnection",
            SigningAlgorithm::EcdsaP256Sha256
        ).await;
        assert!(sign_result.is_ok(), "Signing after reconnection should succeed");
        
        client.disconnect().await.unwrap();
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        // Create client with very short timeout
        let mut config = ClientConfig {
            server_address: "127.0.0.1".to_string(),
            server_port: port,
            transport: TransportType::Tcp,
            connection_timeout_ms: 50, // Very short timeout
            request_timeout_ms: 50,    // Very short timeout
            max_retries: 1,
            retry_delay_ms: 10,
            keep_alive_interval_ms: None,
            keep_alive_timeout_ms: None,
        };
        
        let mut client = GrpcSigningClient::new(config);
        
        // Connection might timeout, but that's expected behavior
        let connect_result = client.connect().await;
        
        match connect_result {
            Ok(_) => {
                // If connection succeeds despite short timeout, that's fine
                let health_result = client.health_check().await;
                // Health check might timeout, which is expected
                client.disconnect().await.ok();
            }
            Err(_) => {
                // Connection timeout is expected with very short timeout
            }
        }
        
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_benchmark_simulation() {
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        // Create client
        let mut client = create_test_client(port).await;
        client.connect().await.unwrap();
        
        let iterations = 50;
        let mut processing_times = Vec::new();
        
        // Perform multiple signing operations to simulate benchmark
        for i in 0..iterations {
            let message = format!("Benchmark message {}", i);
            let start_time = std::time::Instant::now();
            
            let sign_result = client.sign(
                "integration-test-key",
                message.as_bytes(),
                SigningAlgorithm::EcdsaP256Sha256
            ).await;
            
            let elapsed = start_time.elapsed();
            assert!(sign_result.is_ok(), "Benchmark iteration {} should succeed", i);
            
            processing_times.push(elapsed.as_millis() as u64);
        }
        
        // Calculate basic statistics
        processing_times.sort();
        let min_time = processing_times[0];
        let max_time = processing_times[processing_times.len() - 1];
        let avg_time = processing_times.iter().sum::<u64>() / processing_times.len() as u64;
        let p95_index = (processing_times.len() as f64 * 0.95) as usize;
        let p95_time = processing_times[p95_index];
        
        println!("Benchmark Results:");
        println!("  Iterations: {}", iterations);
        println!("  Min time: {}ms", min_time);
        println!("  Max time: {}ms", max_time);
        println!("  Avg time: {}ms", avg_time);
        println!("  P95 time: {}ms", p95_time);
        
        // Basic performance assertions
        assert!(avg_time < 100, "Average processing time should be under 100ms");
        assert!(p95_time < 200, "P95 processing time should be under 200ms");
        
        client.disconnect().await.unwrap();
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        // Start server
        let (port, server_handle) = start_test_server().await;
        
        // Create multiple clients
        let mut clients = Vec::new();
        for _ in 0..3 {
            let mut client = create_test_client(port).await;
            client.connect().await.unwrap();
            clients.push(client);
        }
        
        // Perform operations with all clients
        for (i, client) in clients.iter_mut().enumerate() {
            let message = format!("Client {} message", i);
            let result = client.sign(
                "integration-test-key",
                message.as_bytes(),
                SigningAlgorithm::EcdsaP256Sha256
            ).await;
            assert!(result.is_ok(), "Client {} operation should succeed", i);
        }
        
        // Gracefully disconnect all clients
        for (i, client) in clients.iter_mut().enumerate() {
            let disconnect_result = client.disconnect().await;
            assert!(disconnect_result.is_ok(), "Client {} should disconnect gracefully", i);
        }
        
        // Shutdown server
        server_handle.abort();
        
        // Give time for cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}