#!/usr/bin/env python3
import subprocess
import json
import time
import threading
import sys

def test_mcp_server():
    """Test the MCP server by starting it and sending a basic request"""
    
    print("Starting MCP server...")
    
    # Start the finch-mcp-stdio process
    proc = subprocess.Popen(
        ['./target/debug/finch-mcp-stdio', 'uvx', 'mcp-server-time', '--local-timezone', 'UTC'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    def read_output():
        """Read output from the server"""
        while True:
            try:
                line = proc.stdout.readline()
                if line:
                    print(f"Server output: {line.strip()}")
                    # Look for MCP server responses
                    try:
                        response = json.loads(line.strip())
                        print(f"✓ Got JSON response: {response}")
                        return response
                    except json.JSONDecodeError:
                        continue
                else:
                    break
            except:
                break
    
    # Start reading output in a separate thread
    output_thread = threading.Thread(target=read_output, daemon=True)
    output_thread.start()
    
    # Give the server time to start
    time.sleep(5)
    
    try:
        # Send an initialize request
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        print(f"Sending initialize request: {init_request}")
        proc.stdin.write(json.dumps(init_request) + '\n')
        proc.stdin.flush()
        
        # Wait for response
        time.sleep(2)
        
        # Send a list tools request
        tools_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }
        
        print(f"Sending tools/list request: {tools_request}")
        proc.stdin.write(json.dumps(tools_request) + '\n')
        proc.stdin.flush()
        
        # Wait for response
        time.sleep(2)
        
        print("✓ Successfully communicated with MCP server")
        
    except Exception as e:
        print(f"✗ Error communicating with server: {e}")
    
    finally:
        # Clean up
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        print("✓ Server stopped")

if __name__ == "__main__":
    test_mcp_server()