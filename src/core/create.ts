import path from 'path';
import fs from 'fs-extra';
import chalk from 'chalk';
import ora from 'ora';
import { FinchClient, FinchBuildOptions } from '../finch/finch-client.js';
import { detectPackageManager, PackageManager } from '../utils/package-manager.js';
import { McpDetector } from '../utils/mcp-detector.js';
import { generateDockerfile } from '../utils/template-processor.js';

/**
 * Options for creating an MCP container
 */
interface CreateOptions {
  tag: string;
  name?: string;
}

/**
 * Creates an MCP container from a local repository
 * @param repoPath Path to the MCP server repository
 * @param options Container options
 */
export async function createMcpContainer(repoPath: string, options: CreateOptions): Promise<void> {
  // Resolve absolute path
  const absoluteRepoPath = path.resolve(repoPath);
  
  // Check if the repository exists
  if (!await fs.pathExists(absoluteRepoPath)) {
    throw new Error(`Repository not found at ${absoluteRepoPath}`);
  }
  
  // Create spinner for loading indication
  const spinner = ora('Analyzing MCP server repository...').start();
  
  try {
    // Detect package manager
    const packageManagerInfo = await detectPackageManager(absoluteRepoPath);
    if (!packageManagerInfo) {
      spinner.fail('No package.json found. The repository does not appear to be a Node.js project.');
      return;
    }
    
    // Detect MCP server
    const mcpDetector = new McpDetector();
    const mcpServerInfo = await mcpDetector.detectMcpServer(absoluteRepoPath);
    
    if (!mcpServerInfo) {
      spinner.fail('No MCP server detected in the repository.');
      return;
    }
    
    spinner.succeed('MCP server detected successfully');
    
    // Generate server info
    const serverInfo = `
    Server Type: ${mcpServerInfo.type}
    Entry Point: ${mcpServerInfo.entryPoint}
    Port: ${mcpServerInfo.port || 3000}
    `;
    console.log(chalk.green('MCP Server Information:'));
    console.log(chalk.gray(serverInfo));
    
    // Create a temporary directory for Docker build
    const buildDirName = '.finch-mcp-build';
    const buildDir = path.join(process.cwd(), buildDirName);
    await fs.ensureDir(buildDir);
    
    try {
      // Generate Dockerfile
      spinner.start('Generating Dockerfile...');
      
      // Select the appropriate Dockerfile template based on the package manager
      const dockerfileTemplate = packageManagerInfo.type === PackageManager.NPM 
        ? 'Dockerfile.npm.template' 
        : 'Dockerfile.uv.template';
      
      // Variables for template replacement
      const variables = {
        PORT: String(mcpServerInfo.port || 3000),
        ENTRY_POINT: mcpServerInfo.entryPoint
      };
      
      // Generate and write the Dockerfile
      const dockerfilePath = path.join(buildDir, 'Dockerfile');
      await generateDockerfile(dockerfilePath, dockerfileTemplate, variables);
      
      // Copy the repository files
      await fs.copy(absoluteRepoPath, buildDir, {
        filter: (src: string) => {
          // Exclude node_modules, git, and other unnecessary files
          const relativePath = path.relative(absoluteRepoPath, src);
          return !relativePath.includes('node_modules') && 
                 !relativePath.includes('.git') && 
                 !relativePath.includes('dist') &&
                 !relativePath.includes('.finch-mcp-build');
        }
      });
      
      spinner.succeed('Dockerfile generated and build context prepared');
      
      // Use Finch to build the image
      spinner.start('Building container image with Finch...');
      
      // Initialize Finch client
      const finchClient = new FinchClient();
      
      // Check if Finch is available
      if (!await finchClient.isFinchAvailable()) {
        spinner.fail('Finch is not installed or not available. Please install Finch from https://runfinch.com/');
        return;
      }

      // Determine image name
      const imageName = options.name || path.basename(absoluteRepoPath);
      
      // Build options
      const buildOptions: FinchBuildOptions = {
        tag: `${imageName}:${options.tag}`,
        contextDir: buildDir,
        dockerfilePath: dockerfilePath
      };
      
      // Build the image
      await finchClient.buildImage(buildOptions);
      
      spinner.succeed(`MCP server container image created successfully: ${imageName}:${options.tag}`);
      
      // Print usage instructions
      const usageInstructions = `
      You can now run your MCP server container with:
      
      finch-mcp run ${imageName}:${options.tag}
      
      Or for development with live reloading:
      
      finch-mcp dev ${absoluteRepoPath}
      `;
      
      console.log(chalk.green('Usage Instructions:'));
      console.log(chalk.gray(usageInstructions));
      
    } finally {
      // Clean up the build directory
      await fs.remove(buildDir);
    }
  } catch (error) {
    spinner.fail(`Failed to create MCP container: ${error instanceof Error ? error.message : String(error)}`);
    throw error;
  }
}