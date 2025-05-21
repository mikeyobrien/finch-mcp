import fs from 'fs-extra';
import path from 'path';
import { glob } from 'glob';

/**
 * MCP Server type
 */
export enum McpServerType {
  EXPRESS = 'express',
  FASTIFY = 'fastify',
  CUSTOM = 'custom'
}

/**
 * MCP Server information
 */
export interface McpServerInfo {
  type: McpServerType;
  entryPoint: string;
  port?: number;
  packageJson: Record<string, any>;
  hasMcpDependency: boolean;
}

/**
 * Detects MCP servers in a repository
 */
export class McpDetector {
  /**
   * Detects MCP servers in a repository
   * @param repoPath Path to the repository
   * @returns Information about the detected MCP server
   */
  async detectMcpServer(repoPath: string): Promise<McpServerInfo | null> {
    try {
      // Check if package.json exists
      const packageJsonPath = path.join(repoPath, 'package.json');
      if (!await fs.pathExists(packageJsonPath)) {
        return null;
      }

      // Read package.json
      const packageJson = await fs.readJson(packageJsonPath);
      
      // Check if the project has MCP-related dependencies
      const hasMcpDependency = this.hasMcpDependencies(packageJson);
      
      // Find entry point
      const entryPoint = await this.findEntryPoint(repoPath, packageJson);
      if (!entryPoint) {
        return null;
      }

      // Determine server type
      const type = await this.determineServerType(repoPath, packageJson);
      
      // Try to determine port
      const port = await this.detectPort(repoPath, entryPoint);
      
      return {
        type,
        entryPoint,
        port,
        packageJson,
        hasMcpDependency
      };
    } catch (error) {
      console.error('Error detecting MCP server:', error);
      return null;
    }
  }

  /**
   * Checks if a project has MCP-related dependencies
   * @param packageJson package.json contents
   * @returns true if the project has MCP-related dependencies
   */
  private hasMcpDependencies(packageJson: Record<string, any>): boolean {
    const deps = {
      ...packageJson.dependencies,
      ...packageJson.devDependencies
    };

    // Check for common MCP-related dependencies
    return Object.keys(deps).some(dep => 
      dep.includes('mcp') || 
      dep.includes('ai-plugin') || 
      dep.includes('claude-plugin'));
  }

  /**
   * Finds the entry point for an MCP server
   * @param repoPath Repository path
   * @param packageJson package.json contents
   * @returns Path to the entry point
   */
  private async findEntryPoint(repoPath: string, packageJson: Record<string, any>): Promise<string | null> {
    // First check package.json main field
    if (packageJson.main) {
      const mainPath = path.join(repoPath, packageJson.main);
      if (await fs.pathExists(mainPath)) {
        return packageJson.main;
      }
    }

    // Check common server file names
    const commonServerFiles = [
      'server.js', 'app.js', 'index.js',
      'server.ts', 'app.ts', 'index.ts',
      'src/server.js', 'src/app.js', 'src/index.js',
      'src/server.ts', 'src/app.ts', 'src/index.ts'
    ];

    for (const file of commonServerFiles) {
      const filePath = path.join(repoPath, file);
      if (await fs.pathExists(filePath)) {
        return file;
      }
    }

    // Look for files containing MCP-related code
    const files = await glob('**/*.{js,ts}', { 
      cwd: repoPath,
      ignore: ['**/node_modules/**', '**/dist/**', '**/build/**', '**/*.test.{js,ts}', '**/*.spec.{js,ts}'] 
    });

    for (const file of files) {
      const filePath = path.join(repoPath, file);
      const content = await fs.readFile(filePath, 'utf-8');
      
      // Check for MCP-related patterns
      if (content.includes('mcp') && 
         (content.includes('server') || content.includes('app.listen') || content.includes('createServer'))) {
        return file;
      }
    }

    return null;
  }

  /**
   * Determines the server type
   * @param repoPath Repository path
   * @param packageJson package.json contents
   * @returns Server type
   */
  private async determineServerType(repoPath: string, packageJson: Record<string, any>): Promise<McpServerType> {
    const deps = {
      ...packageJson.dependencies,
      ...packageJson.devDependencies
    };

    if (deps.express) {
      return McpServerType.EXPRESS;
    }

    if (deps.fastify) {
      return McpServerType.FASTIFY;
    }

    return McpServerType.CUSTOM;
  }

  /**
   * Detects the port used by the server
   * @param repoPath Repository path
   * @param entryPoint Entry point file
   * @returns Port number if found, undefined otherwise
   */
  private async detectPort(repoPath: string, entryPoint: string): Promise<number | undefined> {
    try {
      const filePath = path.join(repoPath, entryPoint);
      const content = await fs.readFile(filePath, 'utf-8');
      
      // Look for common port patterns
      const portMatches = [
        // Look for literal port assignments
        /(?:PORT|port)\s*=\s*(\d+)/,
        // Look for listen calls with port
        /\.listen\(\s*(\d+)/,
        // Look for port in environment variables with default
        /process\.env\.PORT\s*\|\|\s*(\d+)/
      ];

      for (const pattern of portMatches) {
        const match = content.match(pattern);
        if (match && match[1]) {
          return parseInt(match[1], 10);
        }
      }

      // Default port for many Node.js servers
      return 3000;
    } catch (error) {
      return undefined;
    }
  }
}