import path from 'path';
import fs from 'fs-extra';
import chalk from 'chalk';
import ora from 'ora';
import { FinchClient, FinchRunOptions } from '../finch/finch-client.js';
import { McpDetector } from '../utils/mcp-detector.js';
import { createMcpContainer } from './create.js';

/**
 * Options for running an MCP container
 */
interface RunOptions {
  port: string;
  env?: string[];
  stdio?: boolean;
  volume?: string[];
}

/**
 * Runs an MCP server container
 * @param pathOrImage Path to the MCP server repository or image name
 * @param options Run options
 */
export async function runMcpContainer(pathOrImage: string, options: RunOptions): Promise<void> {
  const spinner = ora('Preparing to run MCP server container...').start();
  
  try {
    // Initialize Finch client
    const finchClient = new FinchClient();
    
    // Check if Finch is available
    if (!await finchClient.isFinchAvailable()) {
      spinner.fail('Finch is not installed or not available. Please install Finch from https://runfinch.com/');
      return;
    }
    
    // Determine if pathOrImage is a local path or an image name
    let imageName = pathOrImage;
    const absolutePath = path.resolve(pathOrImage);
    
    // Check if the path exists and is a directory
    if (await fs.pathExists(absolutePath) && (await fs.stat(absolutePath)).isDirectory()) {
      spinner.text = 'Local repository detected, building container...';
      
      // Detect MCP server
      const mcpDetector = new McpDetector();
      const mcpServerInfo = await mcpDetector.detectMcpServer(absolutePath);
      
      if (!mcpServerInfo) {
        spinner.fail('No MCP server detected in the repository.');
        return;
      }
      
      // Build container from local repository
      await createMcpContainer(absolutePath, { tag: 'latest' });
      
      // Set image name to the repository name
      imageName = `${path.basename(absolutePath)}:latest`;
    }
    
    spinner.text = `Starting MCP server container ${imageName}...`;
    
    // Prepare run options
    const containerName = `mcp-${path.basename(imageName).replace(/[^a-zA-Z0-9]/g, '-')}`;
    
    // Map port
    const ports = [`${options.port}:${options.port}`];
    
    // Add MCP-specific environment variables
    const envs = [
      ...(options.env || []),
      `PORT=${options.port}`,
      'MCP_ENABLED=true'
    ];
    
    if (options.stdio) {
      // For STDIO mode, run in foreground without port mapping
      spinner.succeed('Starting MCP server in STDIO mode...');
      console.log(chalk.green('\nConnecting to MCP Server...'));
      
      try {
        // Execute with stdio: 'inherit' to directly pipe I/O to the parent process
        const args = ['run', '--rm', '-i', '-e', 'MCP_ENABLED=true', '-e', 'MCP_STDIO=true'];
        
        // Add environment variables
        if (options.env && options.env.length > 0) {
          for (const env of options.env) {
            args.push('-e', env);
          }
        }
        
        // Add volume mounts if specified
        if (options.volume && options.volume.length > 0) {
          for (const vol of options.volume) {
            args.push('-v', vol);
          }
        }
        
        // Add the image name
        args.push(imageName);
        
        // Execute and pipe stdio
        const execProcess = execa('finch', args, {
          stdio: 'inherit'
        });
        
        // Let the process run (interact directly with stdio)
        await execProcess;
      } catch (error) {
        console.error(chalk.red(`STDIO process terminated: ${error instanceof Error ? error.message : String(error)}`));
      }
    } else {
      // Standard HTTP mode
      // Run options
      const runOptions: FinchRunOptions = {
        imageName,
        containerName,
        ports,
        envs,
        detach: true,
        remove: false,
        volumes: options.volume
      };
      
      // Run the container
      await finchClient.runContainer(runOptions);
      
      spinner.succeed(`MCP server container is running at http://localhost:${options.port}`);
    }
    
    // Print additional information
    console.log(chalk.green('\nMCP Server Information:'));
    console.log(chalk.gray(`Container Name: ${containerName}`));
    console.log(chalk.gray(`Exposed Port: ${options.port}`));
    console.log(chalk.gray('Environment Variables:'));
    envs.forEach(env => {
      console.log(chalk.gray(`  - ${env}`));
    });
    
    console.log(chalk.green('\nTo stop the container, press Ctrl+C'));
    
    // Keep the process running until Ctrl+C
    process.on('SIGINT', async () => {
      console.log(chalk.yellow('\nStopping MCP server container...'));
      try {
        await execa('finch', ['stop', containerName]);
        console.log(chalk.green('MCP server container stopped successfully'));
      } catch (error) {
        console.error(chalk.red(`Failed to stop container: ${error instanceof Error ? error.message : String(error)}`));
      } finally {
        process.exit(0);
      }
    });
  } catch (error) {
    spinner.fail(`Failed to run MCP container: ${error instanceof Error ? error.message : String(error)}`);
    throw error;
  }
}

// Import execa at the end to avoid hoisting issues with ESM
import { execa } from 'execa';