import path from 'path';
import fs from 'fs-extra';
import chalk from 'chalk';
import ora from 'ora';
import { FinchClient, FinchBuildOptions, FinchRunOptions } from '../finch/finch-client.js';
import { detectPackageManager, PackageManager, getInstallCommand } from '../utils/package-manager.js';
import { McpDetector } from '../utils/mcp-detector.js';
import { generateDockerfile } from '../utils/template-processor.js';

/**
 * Options for running an MCP container in development mode
 */
interface DevOptions {
  port: string;
  env?: string[];
}

/**
 * Runs an MCP server in development mode with live reloading
 * @param repoPath Path to the MCP server repository
 * @param options Development options
 */
export async function devMcpContainer(repoPath: string, options: DevOptions): Promise<void> {
  // Resolve absolute path
  const absoluteRepoPath = path.resolve(repoPath);
  
  // Check if the repository exists
  if (!await fs.pathExists(absoluteRepoPath)) {
    throw new Error(`Repository not found at ${absoluteRepoPath}`);
  }
  
  // Create spinner for loading indication
  const spinner = ora('Setting up development environment...').start();
  
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
    
    // Create a temporary directory for Docker build
    const buildDir = path.join(absoluteRepoPath, '.finch-mcp-dev');
    await fs.ensureDir(buildDir);
    
    try {
      // Generate Dockerfile for development
      spinner.text = 'Generating development Dockerfile...';
      
      // Variables for template replacement
      const installCommand = getInstallCommand(packageManagerInfo.type);
      const lockfile = packageManagerInfo.type === PackageManager.NPM 
        ? 'package-lock.json' 
        : 'uv-lock.json';
      
      const variables = {
        PORT: String(mcpServerInfo.port || 3000),
        ENTRY_POINT: mcpServerInfo.entryPoint,
        INSTALL_COMMAND: installCommand,
        LOCKFILE: lockfile
      };
      
      // Generate and write the Dockerfile
      const dockerfilePath = path.join(buildDir, 'Dockerfile');
      await generateDockerfile(dockerfilePath, 'Dockerfile.dev.template', variables);
      
      // Copy package files for build context
      await fs.copy(
        path.join(absoluteRepoPath, 'package.json'), 
        path.join(buildDir, 'package.json')
      );
      
      // Copy lock file if it exists
      const lockFilePath = path.join(absoluteRepoPath, lockfile);
      if (await fs.pathExists(lockFilePath)) {
        await fs.copy(lockFilePath, path.join(buildDir, lockfile));
      }
      
      spinner.succeed('Development environment prepared');
      
      // Initialize Finch client
      const finchClient = new FinchClient();
      
      // Check if Finch is available
      if (!await finchClient.isFinchAvailable()) {
        throw new Error('Finch is not installed or not available. Please install Finch from https://runfinch.com/');
      }
      
      // Build the development image
      spinner.start('Building development container...');
      
      // Image name
      const imageName = `${path.basename(absoluteRepoPath)}-dev`;
      
      // Build options
      const buildOptions: FinchBuildOptions = {
        tag: `${imageName}:latest`,
        contextDir: buildDir,
        dockerfilePath: dockerfilePath
      };
      
      // Build the image
      await finchClient.buildImage(buildOptions);
      
      spinner.succeed('Development container built successfully');
      
      // Run the container with volume mount for live reloading
      spinner.start('Starting development container...');
      
      // Prepare run options
      const containerName = `mcp-dev-${path.basename(absoluteRepoPath).replace(/[^a-zA-Z0-9]/g, '-')}`;
      
      // Map port
      const ports = [`${options.port}:${options.port}`];
      
      // Add MCP-specific environment variables
      const envs = [
        ...(options.env || []),
        `PORT=${options.port}`,
        'MCP_ENABLED=true',
        'NODE_ENV=development'
      ];
      
      // Volume mount for live reloading
      const volumes = [`${absoluteRepoPath}:/app`];
      
      // Run options
      const runOptions: FinchRunOptions = {
        imageName: `${imageName}:latest`,
        containerName,
        ports,
        envs,
        volumes,
        detach: true,
        remove: true
      };
      
      // Run the container
      await finchClient.runContainer(runOptions);
      
      spinner.succeed(`MCP server development container is running at http://localhost:${options.port}`);
      
      // Print development information
      console.log(chalk.green('\nDevelopment Mode Information:'));
      console.log(chalk.gray(`Container Name: ${containerName}`));
      console.log(chalk.gray(`Exposed Port: ${options.port}`));
      console.log(chalk.gray(`Repository Path: ${absoluteRepoPath}`));
      console.log(chalk.gray('Live Reloading: Enabled'));
      
      console.log(chalk.green('\nTo stop the development container, press Ctrl+C'));
      
      // Keep the process running until Ctrl+C
      process.on('SIGINT', async () => {
        console.log(chalk.yellow('\nStopping development container...'));
        try {
          await execa('finch', ['stop', containerName]);
          console.log(chalk.green('Development container stopped successfully'));
        } catch (error) {
          console.error(chalk.red(`Failed to stop container: ${error instanceof Error ? error.message : String(error)}`));
        } finally {
          process.exit(0);
        }
      });
    } finally {
      // Clean up the build directory
      await fs.remove(buildDir);
    }
  } catch (error) {
    spinner.fail(`Failed to run development mode: ${error instanceof Error ? error.message : String(error)}`);
    throw error;
  }
}

// Import execa at the end to avoid hoisting issues with ESM
import { execa } from 'execa';