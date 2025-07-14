//! Integration tests for the transport layer supporting TCP and VSOCK

use grpc_performance_rs::transport::{TransportConfig, TransportFactory};
use std::time::Duration;
use tokio::time::sleep;

/// Check if VSOCK is available in the test environment
async fn is_vsock_available() -> bool {
    let config = TransportConfig::Vsock { cid: 2, port: 50052 };
    match TransportFactory::connect(&config).await {
        Ok(_) => true,
        Err(e) => {
            println!("VSOCK not available: {}", e);
            false
        }
    }
}

#[tokio::test]
async fn test_tcp_transport_config_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing TCP transport configuration parsing");
    
    // Test TCP address parsing
    let tcp_config: TransportConfig = "127.0.0.1:50051".parse()?;
    assert!(tcp_config.is_tcp());
    assert_eq!(tcp_config.port(), 50051);
    assert_eq!(tcp_config.to_string(), "127.0.0.1:50051");
    
    println!("✅ TCP transport configuration parsing tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_vsock_transport_config_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing VSOCK transport configuration parsing");
    
    // Test VSOCK address parsing
    let vsock_config: TransportConfig = "vsock://2:50051".parse()?;
    assert!(vsock_config.is_vsock());
    assert_eq!(vsock_config.port(), 50051);
    assert_eq!(vsock_config.to_string(), "vsock://2:50051");
    
    if let TransportConfig::Vsock { cid, port } = vsock_config {
        assert_eq!(cid, 2);
        assert_eq!(port, 50051);
    } else {
        panic!("Expected VSOCK config");
    }
    
    println!("✅ VSOCK transport configuration parsing tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_invalid_transport_config_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing invalid transport configuration parsing");
    
    // Test invalid addresses
    assert!("invalid".parse::<TransportConfig>().is_err());
    assert!("vsock://invalid:port".parse::<TransportConfig>().is_err());
    assert!("vsock://2".parse::<TransportConfig>().is_err());
    assert!("vsock://2:invalid".parse::<TransportConfig>().is_err());
    assert!("256.256.256.256:50051".parse::<TransportConfig>().is_err());
    
    println!("✅ Invalid transport configuration parsing tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_transport_factory_names() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing transport factory names");
    
    // Test TCP transport name
    let tcp_config: TransportConfig = "127.0.0.1:50051".parse()?;
    assert_eq!(TransportFactory::transport_name(&tcp_config), "TCP");
    
    // Test VSOCK transport name
    let vsock_config: TransportConfig = "vsock://2:50051".parse()?;
    assert_eq!(TransportFactory::transport_name(&vsock_config), "VSOCK");
    
    println!("✅ Transport factory name tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_tcp_transport_bind_and_connect() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing TCP transport bind and connect");
    
    // Test TCP binding
    let tcp_config: TransportConfig = "127.0.0.1:0".parse()?; // Use port 0 for automatic assignment
    let mut listener = TransportFactory::bind(&tcp_config).await?;
    
    let local_addr = listener.local_addr()?;
    println!("TCP listener bound to: {}", local_addr);
    
    // Extract the actual port that was assigned
    let actual_port = if let Ok(addr) = local_addr.parse::<std::net::SocketAddr>() {
        addr.port()
    } else {
        return Err("Failed to parse local address".into());
    };
    
    // Test TCP connection in a separate task
    let connect_config: TransportConfig = format!("127.0.0.1:{}", actual_port).parse()?;
    
    // Spawn a task to accept the connection
    let accept_handle = tokio::spawn(async move {
        match listener.accept().await {
            Ok(connection) => {
                println!("Accepted TCP connection from: {:?}", connection.remote_addr());
                Ok(())
            }
            Err(e) => Err(e)
        }
    });
    
    // Give the listener a moment to be ready
    sleep(Duration::from_millis(10)).await;
    
    // Connect to the listener
    let connection = TransportFactory::connect(&connect_config).await?;
    println!("TCP connection established to: {:?}", connection.remote_addr());
    
    // Wait for the accept to complete
    accept_handle.await??;
    
    println!("✅ TCP transport bind and connect tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_vsock_transport_availability() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing VSOCK transport availability");
    
    if !is_vsock_available().await {
        println!("VSOCK not available in test environment, skipping VSOCK transport tests");
        return Ok(());
    }
    
    println!("VSOCK is available in test environment");
    
    // Test VSOCK binding (this might fail if we don't have proper permissions)
    let vsock_config = TransportConfig::Vsock { cid: 2, port: 50052 };
    
    match TransportFactory::bind(&vsock_config).await {
        Ok(listener) => {
            println!("VSOCK listener bound to: {:?}", listener.local_addr());
            println!("✅ VSOCK transport bind test passed!");
        }
        Err(e) => {
            println!("VSOCK bind failed (expected in some environments): {}", e);
            // This is not necessarily a failure - VSOCK might not be available or we might not have permissions
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_tcp_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing TCP error handling");
    
    // Test connection to unreachable TCP address should fail
    // Use a port that's likely to be closed
    let config: TransportConfig = "127.0.0.1:1".parse()?;
    let result = TransportFactory::connect(&config).await;
    assert!(result.is_err(), "Connection to unreachable TCP address should fail");
    
    // Test binding to invalid address
    let invalid_config = "256.256.256.256:50051".parse::<TransportConfig>();
    assert!(invalid_config.is_err(), "Invalid IP address should fail to parse");
    
    println!("✅ TCP error handling tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_vsock_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    if !is_vsock_available().await {
        println!("VSOCK not available in test environment, skipping VSOCK error handling tests");
        return Ok(());
    }
    
    println!("Testing VSOCK error handling");
    
    // Test connection to invalid VSOCK address should fail
    let config = TransportConfig::Vsock { cid: 999, port: 50052 };
    let result = TransportFactory::connect(&config).await;
    assert!(result.is_err(), "Connection to invalid VSOCK address should fail");
    
    println!("✅ VSOCK error handling tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_transport_config_properties() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing transport configuration properties");
    
    // Test TCP config properties
    let tcp_config: TransportConfig = "192.168.1.100:8080".parse()?;
    assert!(tcp_config.is_tcp());
    assert!(!tcp_config.is_vsock());
    assert_eq!(tcp_config.port(), 8080);
    
    // Test VSOCK config properties
    let vsock_config = TransportConfig::Vsock { cid: 3, port: 9090 };
    assert!(!vsock_config.is_tcp());
    assert!(vsock_config.is_vsock());
    assert_eq!(vsock_config.port(), 9090);
    
    println!("✅ Transport configuration property tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_multiple_tcp_connections() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing multiple TCP connections");
    
    // Bind a TCP listener
    let tcp_config: TransportConfig = "127.0.0.1:0".parse()?;
    let mut listener = TransportFactory::bind(&tcp_config).await?;
    
    let local_addr = listener.local_addr()?;
    let actual_port = if let Ok(addr) = local_addr.parse::<std::net::SocketAddr>() {
        addr.port()
    } else {
        return Err("Failed to parse local address".into());
    };
    
    let connect_config: TransportConfig = format!("127.0.0.1:{}", actual_port).parse()?;
    
    // Spawn a task to accept multiple connections
    let accept_handle = tokio::spawn(async move {
        for i in 0..3 {
            match listener.accept().await {
                Ok(connection) => {
                    println!("Accepted TCP connection #{} from: {:?}", i + 1, connection.remote_addr());
                }
                Err(e) => return Err(e)
            }
        }
        Ok(())
    });
    
    // Give the listener a moment to be ready
    sleep(Duration::from_millis(10)).await;
    
    // Create multiple connections
    for i in 0..3 {
        let connection = TransportFactory::connect(&connect_config).await?;
        println!("TCP connection #{} established to: {:?}", i + 1, connection.remote_addr());
        
        // Small delay between connections
        sleep(Duration::from_millis(10)).await;
    }
    
    // Wait for all accepts to complete
    accept_handle.await??;
    
    println!("✅ Multiple TCP connection tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_grpc_service_integration_placeholder() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing gRPC service integration (placeholder)");
    
    // This is a placeholder test that demonstrates how we would test
    // the actual gRPC services once they are implemented with the transport layer
    
    // The echo and crypto modules are available from the proto generation
    use grpc_performance_rs::echo::EchoRequest;
    use grpc_performance_rs::crypto::{SignRequest, PublicKeyRequest, KeyType, SigningAlgorithm};
    
    // Test that we can create the proto message types
    let echo_req = EchoRequest {
        payload: "test".to_string(),
        timestamp: 1234567890,
    };
    assert_eq!(echo_req.payload, "test");
    
    let sign_req = SignRequest {
        data: b"test data".to_vec(),
        key_type: KeyType::Rsa as i32,
        algorithm: SigningAlgorithm::RsaPkcs1Sha256 as i32,
        timestamp: 1234567890,
    };
    assert_eq!(sign_req.data, b"test data");
    
    let pubkey_req = PublicKeyRequest {
        key_type: KeyType::Rsa as i32,
        timestamp: 1234567890,
    };
    assert_eq!(pubkey_req.key_type, KeyType::Rsa as i32);
    
    println!("✅ gRPC proto message types work correctly!");
    
    // TODO: Once the server and client implementations are integrated with the transport layer,
    // we would add tests here that:
    // 1. Start a server with both TCP and VSOCK transports
    // 2. Connect clients using both transports
    // 3. Test that the same service functionality works identically on both transports
    // 4. Test error handling for invalid addresses
    // 5. Ensure proper cleanup for both transport types
    
    Ok(())
}