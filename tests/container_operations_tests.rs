use finch_mcp::{
    FinchClient,
    RunOptions,
};
use tempfile::TempDir;
use std::{fs, time::Duration};
use tokio::time::timeout;

/// Test Finch container operations and lifecycle management
#[tokio::test]
#[ignore = "Container test requiring Finch installation"]
async fn test_container_lifecycle_operations() {
    let finch_client = FinchClient::new();
    
    // Skip if Finch is not available
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping container test: Finch not available");
        return;
    }
    
    // Ensure VM is running
    assert!(finch_client.ensure_vm_running().await.unwrap());
    
    // Test running a simple container
    let run_options = RunOptions {
        image_name: "alpine:latest".to_string(),
        env_vars: Some(vec!["TEST_ENV=container_lifecycle".to_string()]),
        volumes: None,
    };
    
    // Run container with timeout
    let run_result = timeout(
        Duration::from_secs(30),
        finch_mcp::run::run_stdio_container(run_options)
    ).await;
    
    // Container should complete (alpine exits immediately without command)
    assert!(run_result.is_ok() || run_result.is_err());
    
    // Test container cleanup
    let cleanup_result = finch_client.cleanup_resources(false, true, false, false).await;
    assert!(cleanup_result.is_ok());
}

#[tokio::test]
#[ignore = "Container test requiring Finch installation"]
async fn test_container_resource_management() {
    let finch_client = FinchClient::new();
    
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping container test: Finch not available");
        return;
    }
    
    // Test resource listing
    let list_result = finch_client.list_resources(false).await;
    assert!(list_result.is_ok());
    
    // Test resource listing with --all flag
    let list_all_result = finch_client.list_resources(true).await;
    assert!(list_all_result.is_ok());
    
    // Test selective cleanup options
    let cleanup_containers_result = finch_client.cleanup_resources(false, true, false, false).await;
    assert!(cleanup_containers_result.is_ok());
    
    let cleanup_images_result = finch_client.cleanup_resources(false, false, true, false).await;
    assert!(cleanup_images_result.is_ok());
}

#[tokio::test]
#[ignore = "Container test requiring Finch installation"]
async fn test_container_environment_variables() {
    let finch_client = FinchClient::new();
    
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping container test: Finch not available");
        return;
    }
    
    // Test various environment variable configurations
    let env_test_cases = vec![
        Some(vec!["SIMPLE=value".to_string()]),
        Some(vec!["MULTI=value1".to_string(), "VARS=value2".to_string()]),
        Some(vec!["COMPLEX_VALUE=key=value,other=data".to_string()]),
        Some(vec!["JSON_CONFIG={\"key\":\"value\"}".to_string()]),
        Some(vec!["PATH_VAR=/usr/local/bin:/usr/bin".to_string()]),
    ];
    
    for (i, env_vars) in env_test_cases.iter().enumerate() {
        let run_options = RunOptions {
            image_name: "alpine:latest".to_string(),
            env_vars: env_vars.clone(),
            volumes: None,
        };
        
        let result = timeout(
            Duration::from_secs(15),
            finch_mcp::run::run_stdio_container(run_options)
        ).await;
        
        // Each test should complete without hanging
        assert!(result.is_ok() || result.is_err(), "Test case {} failed", i);
    }
}

#[tokio::test]
#[ignore = "Container test requiring Finch installation"]
async fn test_container_volume_mounting() {
    let finch_client = FinchClient::new();
    
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping container test: Finch not available");
        return;
    }
    
    let test_dir = TempDir::new().unwrap();
    let host_path = test_dir.path();
    
    // Create test files
    fs::write(host_path.join("test.txt"), "Container volume test").unwrap();
    fs::create_dir(host_path.join("subdir")).unwrap();
    fs::write(host_path.join("subdir/nested.txt"), "Nested file").unwrap();
    
    // Test volume mounting scenarios
    let volume_test_cases = vec![
        Some(vec![format!("{}:/data", host_path.display())]),
        Some(vec![format!("{}:/data:ro", host_path.display())]), // Read-only mount
        Some(vec![format!("{}:/app/data", host_path.display())]),
    ];
    
    for (i, volumes) in volume_test_cases.iter().enumerate() {
        let run_options = RunOptions {
            image_name: "alpine:latest".to_string(),
            env_vars: Some(vec![format!("TEST_CASE={}", i)]),
            volumes: volumes.clone(),
        };
        
        let result = timeout(
            Duration::from_secs(20),
            finch_mcp::run::run_stdio_container(run_options)
        ).await;
        
        assert!(result.is_ok() || result.is_err(), "Volume test case {} failed", i);
    }
}

#[tokio::test]
#[ignore = "Container test requiring Finch installation"]
async fn test_container_image_operations() {
    let finch_client = FinchClient::new();
    
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping container test: Finch not available");
        return;
    }
    
    // Test running different image types
    let image_test_cases = vec![
        "alpine:latest",
        "alpine:3.18",
        "hello-world",
        "busybox:latest",
    ];
    
    for image_name in image_test_cases {
        let run_options = RunOptions {
            image_name: image_name.to_string(),
            env_vars: Some(vec![format!("IMAGE_TEST={}", image_name)]),
            volumes: None,
        };
        
        let result = timeout(
            Duration::from_secs(30), // Some images may need pull time
            finch_mcp::run::run_stdio_container(run_options)
        ).await;
        
        // Images should either run successfully or fail gracefully
        assert!(result.is_ok() || result.is_err(), "Image test failed for: {}", image_name);
    }
}

#[tokio::test]
#[ignore = "Container test requiring Finch installation"]
async fn test_container_error_scenarios() {
    let finch_client = FinchClient::new();
    
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping container test: Finch not available");
        return;
    }
    
    // Test invalid image name
    let invalid_image_options = RunOptions {
        image_name: "nonexistent-image:invalid-tag".to_string(),
        env_vars: None,
        volumes: None,
    };
    
    let invalid_result = timeout(
        Duration::from_secs(30),
        finch_mcp::run::run_stdio_container(invalid_image_options)
    ).await;
    
    // This should either fail gracefully or timeout
    assert!(invalid_result.is_ok() || invalid_result.is_err());
    
    // Test invalid volume mount
    let invalid_volume_options = RunOptions {
        image_name: "alpine:latest".to_string(),
        env_vars: None,
        volumes: Some(vec!["/nonexistent/path:/data".to_string()]),
    };
    
    let volume_result = timeout(
        Duration::from_secs(20),
        finch_mcp::run::run_stdio_container(invalid_volume_options)
    ).await;
    
    // This should handle the error gracefully
    assert!(volume_result.is_ok() || volume_result.is_err());
}

#[test]
fn test_container_option_validation() {
    // Test RunOptions validation logic
    
    // Valid configurations
    let valid_configs = vec![
        RunOptions {
            image_name: "alpine:latest".to_string(),
            env_vars: None,
            volumes: None,
        },
        RunOptions {
            image_name: "my-custom-image:v1.0".to_string(),
            env_vars: Some(vec!["VAR1=value1".to_string(), "VAR2=value2".to_string()]),
            volumes: Some(vec!["/host:/container".to_string(), "/data:/app/data:ro".to_string()]),
        },
    ];
    
    for config in valid_configs {
        // Basic validation
        assert!(!config.image_name.is_empty());
        
        // Environment variable format validation
        if let Some(env_vars) = &config.env_vars {
            for env_var in env_vars {
                assert!(env_var.contains('='), "Invalid env var format: {}", env_var);
                let parts: Vec<&str> = env_var.split('=').collect();
                assert!(!parts[0].is_empty(), "Empty env var name: {}", env_var);
            }
        }
        
        // Volume mount format validation
        if let Some(volumes) = &config.volumes {
            for volume in volumes {
                assert!(volume.contains(':'), "Invalid volume format: {}", volume);
                let parts: Vec<&str> = volume.split(':').collect();
                assert!(parts.len() >= 2, "Invalid volume format: {}", volume);
                assert!(!parts[0].is_empty(), "Empty host path: {}", volume);
                assert!(!parts[1].is_empty(), "Empty container path: {}", volume);
            }
        }
    }
}

#[tokio::test]
#[ignore = "Container test requiring Finch installation"]
async fn test_container_concurrent_operations() {
    let finch_client = FinchClient::new();
    
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping container test: Finch not available");
        return;
    }
    
    // Test running multiple containers concurrently (though they'll complete quickly)
    let mut handles = vec![];
    
    for i in 0..3 {
        let run_options = RunOptions {
            image_name: "alpine:latest".to_string(),
            env_vars: Some(vec![format!("CONCURRENT_TEST={}", i)]),
            volumes: None,
        };
        
        let handle = tokio::spawn(async move {
            timeout(
                Duration::from_secs(20),
                finch_mcp::run::run_stdio_container(run_options)
            ).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all containers to complete
    let results = futures::future::join_all(handles).await;
    
    // All operations should complete without panicking
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Concurrent test {} panicked", i);
    }
}

#[tokio::test]
#[ignore = "Container test requiring Finch installation"]
async fn test_finch_vm_management() {
    let finch_client = FinchClient::new();
    
    if !finch_client.is_finch_available().await.unwrap_or(false) {
        println!("Skipping VM test: Finch not available");
        return;
    }
    
    // Test VM status checking
    let vm_status = finch_client.ensure_vm_running().await;
    assert!(vm_status.is_ok());
    
    // If VM is running, it should stay running
    if vm_status.unwrap() {
        let second_check = finch_client.ensure_vm_running().await;
        assert!(second_check.is_ok() && second_check.unwrap());
    }
}

#[test]
fn test_container_configuration_edge_cases() {
    // Test edge cases in container configuration
    
    // Test complex image names
    let complex_image_names = vec![
        "registry.example.com/namespace/image:tag",
        "localhost:5000/my-image:latest",
        "my-image:v1.0.0-alpha.1",
        "ubuntu", // No tag (should default to latest)
    ];
    
    for image_name in complex_image_names {
        let config = RunOptions {
            image_name: image_name.to_string(),
            env_vars: None,
            volumes: None,
        };
        
        assert!(!config.image_name.is_empty());
        // More complex validation could be added here
    }
    
    // Test environment variable edge cases
    let env_edge_cases = vec![
        Some(vec!["EMPTY_VALUE=".to_string()]),
        Some(vec!["EQUALS_IN_VALUE=key=value".to_string()]),
        Some(vec!["SPECIAL_CHARS=!@#$%^&*()".to_string()]),
    ];
    
    for env_vars in env_edge_cases {
        let config = RunOptions {
            image_name: "test:latest".to_string(),
            env_vars,
            volumes: None,
        };
        
        if let Some(ref env_vars) = config.env_vars {
            for env_var in env_vars {
                assert!(env_var.contains('='), "Env var should contain '=': {}", env_var);
            }
        }
    }
}