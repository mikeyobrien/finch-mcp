import { describe, expect, test, beforeEach, afterEach, jest } from '@jest/globals';
import fs from 'fs-extra';
import path from 'path';
import os from 'os';
import { McpDetector, McpServerType } from '../../src/utils/mcp-detector';

// Mock dependencies
jest.mock('glob');
import glob from 'glob';
const mockedGlob = jest.mocked(glob);

describe('MCP Detector', () => {
  let tempDir: string;
  let mcpDetector: McpDetector;

  beforeEach(async () => {
    // Create a temporary directory for tests
    tempDir = path.join(os.tmpdir(), `finch-mcp-test-${Date.now()}`);
    await fs.mkdir(tempDir, { recursive: true });
    
    // Create an instance of McpDetector
    mcpDetector = new McpDetector();
    
    // Reset all mocks
    jest.resetAllMocks();
  });

  afterEach(async () => {
    // Clean up the temporary directory
    await fs.remove(tempDir);
  });

  describe('detectMcpServer', () => {
    test('should return null if no package.json exists', async () => {
      const result = await mcpDetector.detectMcpServer(tempDir);
      expect(result).toBeNull();
    });

    test('should detect an Express MCP server', async () => {
      // Create a package.json file
      await fs.writeJSON(path.join(tempDir, 'package.json'), {
        name: 'express-mcp-server',
        main: 'server.js',
        dependencies: {
          express: '^4.17.1',
          'mcp-server': '^1.0.0'
        }
      });
      
      // Create a server.js file
      await fs.writeFile(path.join(tempDir, 'server.js'), `
        const express = require('express');
        const app = express();
        const PORT = process.env.PORT || 3000;
        
        app.use('/mcp', require('mcp-server').router);
        
        app.listen(PORT, () => {
          console.log(\`MCP server listening on port \${PORT}\`);
        });
      `);
      
      const result = await mcpDetector.detectMcpServer(tempDir);
      
      expect(result).not.toBeNull();
      expect(result?.type).toBe(McpServerType.EXPRESS);
      expect(result?.entryPoint).toBe('server.js');
      expect(result?.port).toBe(3000);
      expect(result?.hasMcpDependency).toBe(true);
    });

    test('should detect a Fastify MCP server', async () => {
      // Create a package.json file
      await fs.writeJSON(path.join(tempDir, 'package.json'), {
        name: 'fastify-mcp-server',
        main: 'app.js',
        dependencies: {
          fastify: '^3.0.0',
          '@mcp/fastify-plugin': '^1.0.0'
        }
      });
      
      // Create an app.js file
      await fs.writeFile(path.join(tempDir, 'app.js'), `
        const fastify = require('fastify')();
        const mcpPlugin = require('@mcp/fastify-plugin');
        
        fastify.register(mcpPlugin);
        
        fastify.listen(8000, (err) => {
          if (err) throw err;
          console.log('Server listening on port 8000');
        });
      `);
      
      const result = await mcpDetector.detectMcpServer(tempDir);
      
      expect(result).not.toBeNull();
      expect(result?.type).toBe(McpServerType.FASTIFY);
      expect(result?.entryPoint).toBe('app.js');
      expect(result?.port).toBe(8000);
      expect(result?.hasMcpDependency).toBe(true);
    });

    test('should find MCP server in src directory', async () => {
      // Create a package.json file
      await fs.writeJSON(path.join(tempDir, 'package.json'), {
        name: 'custom-mcp-server',
        main: 'src/index.js',
        dependencies: {
          'claude-plugin': '^2.0.0'
        }
      });
      
      // Create src directory
      await fs.mkdir(path.join(tempDir, 'src'), { recursive: true });
      
      // Create an index.js file
      await fs.writeFile(path.join(tempDir, 'src/index.js'), `
        const http = require('http');
        const { createMcpHandler } = require('claude-plugin');
        
        const server = http.createServer((req, res) => {
          if (req.url.startsWith('/mcp')) {
            return createMcpHandler()(req, res);
          }
          
          res.writeHead(200);
          res.end('Hello World');
        });
        
        server.listen(4000);
      `);
      
      mockedGlob.mockResolvedValueOnce(['src/index.js'] as any);
      
      const result = await mcpDetector.detectMcpServer(tempDir);
      
      expect(result).not.toBeNull();
      expect(result?.type).toBe(McpServerType.CUSTOM);
      expect(result?.entryPoint).toBe('src/index.js');
      expect(result?.port).toBe(4000);
      expect(result?.hasMcpDependency).toBe(true);
    });
  });
});