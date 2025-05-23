#!/usr/bin/env node
const { spawn } = require('child_process');
const path = require('path');

// Path to the finch-mcp-stdio binary
const finchMcpPath = path.resolve(__dirname, '../target/debug/finch-mcp-stdio');

// Spawn the finch-mcp-stdio process with the time server command
const proc = spawn(finchMcpPath, ['uvx', 'mcp-server-time', '--local-timezone', 'UTC']);

// Handle process errors
proc.on('error', (err) => {
  console.error('Failed to start finch-mcp-stdio:', err);
  process.exit(1);
});

// Log stderr
proc.stderr.on('data', (data) => {
  console.error(`stderr: ${data}`);
});

// Function to send a command to the MCP server and get the response
async function sendCommand(command) {
  return new Promise((resolve) => {
    // Set up a one-time listener for the response
    const responseListener = (data) => {
      const responseText = data.toString().trim();
      // Look for a JSON response in the output
      try {
        // Find JSON-like content in the response
        const jsonMatch = responseText.match(/\{.*\}/);
        if (jsonMatch) {
          const jsonResponse = JSON.parse(jsonMatch[0]);
          proc.stdout.removeListener('data', responseListener);
          resolve(jsonResponse);
        }
      } catch (e) {
        console.log('Raw output:', responseText);
      }
    };

    // Listen for the response
    proc.stdout.on('data', responseListener);

    // Send the command
    proc.stdin.write(JSON.stringify(command) + '\n');
  });
}

// Main function to test the time server
async function main() {
  console.log('Testing MCP Time Server running in Finch container');
  
  try {
    // Test get_current_time
    console.log('\n1. Testing get_current_time for New York:');
    const getCurrentTimeResponse = await sendCommand({
      name: 'get_current_time',
      arguments: {
        timezone: 'America/New_York'
      }
    });
    console.log('Response:', getCurrentTimeResponse);
    
    // Test convert_time
    console.log('\n2. Testing convert_time from Tokyo to London:');
    const convertTimeResponse = await sendCommand({
      name: 'convert_time',
      arguments: {
        source_timezone: 'Asia/Tokyo',
        time: '14:30',
        target_timezone: 'Europe/London'
      }
    });
    console.log('Response:', convertTimeResponse);
    
    // All tests completed successfully
    console.log('\nAll tests completed successfully!');
    
    // Exit the process
    proc.stdin.end();
    proc.kill();
    process.exit(0);
  } catch (error) {
    console.error('Error during tests:', error);
    proc.kill();
    process.exit(1);
  }
}

// Run the main function
main();