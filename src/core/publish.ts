import path from 'path';
import fs from 'fs-extra';
import chalk from 'chalk';
import ora from 'ora';
import inquirer from 'inquirer';
import { FinchClient, FinchPublishOptions } from '../finch/finch-client.js';
import { McpDetector } from '../utils/mcp-detector.js';
import { createMcpContainer } from './create.js';

/**
 * Options for publishing an MCP container
 */
interface PublishOptions {
  registry?: string;
  tag: string;
  name?: string;
}

/**
 * Publishes an MCP server container to a registry
 * @param repoPath Path to the MCP server repository
 * @param options Publish options
 */
export async function publishMcpContainer(repoPath: string, options: PublishOptions): Promise<void> {
  // Resolve absolute path
  const absoluteRepoPath = path.resolve(repoPath);
  
  // Check if the repository exists
  if (!await fs.pathExists(absoluteRepoPath)) {
    throw new Error(`Repository not found at ${absoluteRepoPath}`);
  }
  
  // Create spinner for loading indication
  const spinner = ora('Analyzing MCP server repository...').start();
  
  try {
    // Detect MCP server
    const mcpDetector = new McpDetector();
    const mcpServerInfo = await mcpDetector.detectMcpServer(absoluteRepoPath);
    
    if (!mcpServerInfo) {
      spinner.fail('No MCP server detected in the repository.');
      return;
    }
    
    spinner.succeed('MCP server detected successfully');
    
    // Determine image name
    const imageName = options.name || path.basename(absoluteRepoPath);
    
    // Prompt for registry if not provided
    let registry = options.registry;
    if (!registry) {
      spinner.stop();
      const answers = await inquirer.prompt([
        {
          type: 'input',
          name: 'registry',
          message: 'Enter container registry URL:',
          default: 'docker.io/username'
        }
      ]);
      registry = answers.registry;
      spinner.start('Building MCP server container...');
    }
    
    // Build container if it doesn't exist
    await createMcpContainer(absoluteRepoPath, { 
      tag: options.tag,
      name: imageName 
    });
    
    // Initialize Finch client
    const finchClient = new FinchClient();
    
    // Check if Finch is available
    if (!await finchClient.isFinchAvailable()) {
      spinner.fail('Finch is not installed or not available. Please install Finch from https://runfinch.com/');
      return;
    }
    
    // Publish the image
    spinner.text = `Publishing MCP server container to ${registry}...`;
    
    // Publish options
    const publishOptions: FinchPublishOptions = {
      imageName,
      registry: registry || '',
      tag: options.tag
    };
    
    // Publish the image
    await finchClient.publishImage(publishOptions);
    
    spinner.succeed(`MCP server container published successfully to ${registry}/${imageName}:${options.tag}`);
    
    // Print installation instructions
    const installInstructions = `
    You can now install and run your MCP server using:
    
    npx ${registry}/${imageName}
    
    Or with specific tag:
    
    npx ${registry}/${imageName}@${options.tag}
    
    If you're using uv:
    
    uvx ${registry}/${imageName}
    `;
    
    console.log(chalk.green('\nInstallation Instructions:'));
    console.log(chalk.gray(installInstructions));
    
  } catch (error) {
    spinner.fail(`Failed to publish MCP container: ${error instanceof Error ? error.message : String(error)}`);
    throw error;
  }
}