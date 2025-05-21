#!/usr/bin/env node

import { Command } from 'commander';
import chalk from 'chalk';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs-extra';

// When using ES modules with Node.js, import.meta.url provides the URL of the current module
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load package.json to get version information
const packageJson = JSON.parse(
  fs.readFileSync(path.join(__dirname, '../package.json'), 'utf8')
);

// Create a new command line program
const program = new Command();

// Set basic program information
program
  .name('finch-mcp')
  .description('Tool for containerizing and distributing MCP servers using Finch')
  .version(packageJson.version);

// Define commands
program
  .command('create')
  .description('Create an MCP server container from a local repository')
  .argument('<path>', 'Path to the MCP server repository')
  .option('-t, --tag <tag>', 'Tag for the container image', 'latest')
  .option('-n, --name <name>', 'Name for the container image')
  .action(async (repoPath, options) => {
    const { createMcpContainer } = await import('./core/create.js');
    try {
      await createMcpContainer(repoPath, options);
    } catch (error) {
      console.error(chalk.red(`Error creating MCP container: ${error instanceof Error ? error.message : String(error)}`));
      process.exit(1);
    }
  });

program
  .command('run')
  .description('Run an MCP server container')
  .argument('<path>', 'Path to the MCP server repository or image name')
  .option('-p, --port <port>', 'Port to expose the MCP server on', '3000')
  .option('-e, --env <env...>', 'Environment variables to pass to the container')
  .option('-v, --volume <volume...>', 'Mount volumes in the container (format: /host/path:/container/path)')
  .option('--stdio', 'Run in STDIO mode for direct pipe communication')
  .action(async (pathOrImage, options) => {
    const { runMcpContainer } = await import('./core/run.js');
    try {
      await runMcpContainer(pathOrImage, options);
    } catch (error) {
      console.error(chalk.red(`Error running MCP container: ${error instanceof Error ? error.message : String(error)}`));
      process.exit(1);
    }
  });

program
  .command('dev')
  .description('Run an MCP server in development mode with live reloading')
  .argument('<path>', 'Path to the MCP server repository')
  .option('-p, --port <port>', 'Port to expose the MCP server on', '3000')
  .option('-e, --env <env...>', 'Environment variables to pass to the container')
  .action(async (repoPath, options) => {
    const { devMcpContainer } = await import('./core/dev.js');
    try {
      await devMcpContainer(repoPath, options);
    } catch (error) {
      console.error(chalk.red(`Error starting development mode: ${error instanceof Error ? error.message : String(error)}`));
      process.exit(1);
    }
  });

program
  .command('publish')
  .description('Publish an MCP server container to a registry')
  .argument('<path>', 'Path to the MCP server repository')
  .option('-r, --registry <registry>', 'Container registry to publish to')
  .option('-t, --tag <tag>', 'Tag for the container image', 'latest')
  .option('-n, --name <name>', 'Name for the container image')
  .action(async (repoPath, options) => {
    const { publishMcpContainer } = await import('./core/publish.js');
    try {
      await publishMcpContainer(repoPath, options);
    } catch (error) {
      console.error(chalk.red(`Error publishing MCP container: ${error instanceof Error ? error.message : String(error)}`));
      process.exit(1);
    }
  });

// Parse command line arguments
program.parse();