import { execa } from 'execa';
import chalk from 'chalk';
import ora from 'ora';

/**
 * Options for building a container image with Finch
 */
export interface FinchBuildOptions {
  tag: string;
  platform?: string;
  buildArgs?: Record<string, string>;
  contextDir: string;
  dockerfilePath?: string;
  noCache?: boolean;
}

/**
 * Options for running a container with Finch
 */
export interface FinchRunOptions {
  imageName: string;
  containerName?: string;
  ports?: string[];
  envs?: string[];
  volumes?: string[];
  detach?: boolean;
  remove?: boolean;
  network?: string;
  interactive?: boolean;
}

/**
 * Options for publishing a container image with Finch
 */
export interface FinchPublishOptions {
  imageName: string;
  registry: string;
  tag: string;
}

/**
 * Client for interacting with Finch container CLI
 */
export class FinchClient {
  /**
   * Checks if Finch is installed and running
   * @returns true if Finch is installed and running, false otherwise
   */
  async isFinchAvailable(): Promise<boolean> {
    try {
      await execa('finch', ['version']);
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Ensures Finch VM is started
   * @returns true if VM is running
   */
  async ensureVmRunning(): Promise<boolean> {
    try {
      const { stdout } = await execa('finch', ['vm', 'status']);
      
      // Check for 'Running' status (case-insensitive)
      if (stdout.toLowerCase().includes('running')) {
        return true;
      }
      
      const spinner = ora('Starting Finch VM...').start();
      await execa('finch', ['vm', 'start']);
      spinner.succeed('Finch VM started successfully');
      return true;
    } catch (error) {
      console.error(chalk.red('Failed to start Finch VM. Please make sure Finch is installed correctly.'));
      return false;
    }
  }

  /**
   * Builds a container image using Finch
   * @param options Build options
   * @returns Promise resolving to the built image ID
   */
  async buildImage(options: FinchBuildOptions): Promise<string> {
    if (!await this.ensureVmRunning()) {
      throw new Error('Finch VM is not running');
    }

    const { tag, contextDir, dockerfilePath, buildArgs = {}, platform, noCache } = options;
    
    const args = ['build', '-t', tag];
    
    // Add optional arguments
    if (dockerfilePath) {
      args.push('-f', dockerfilePath);
    }
    
    if (platform) {
      args.push('--platform', platform);
    }
    
    if (noCache) {
      args.push('--no-cache');
    }
    
    // Add build args
    Object.entries(buildArgs).forEach(([key, value]) => {
      args.push('--build-arg', `${key}=${value}`);
    });
    
    // Add context directory
    args.push(contextDir);
    
    const spinner = ora(`Building image ${tag}...`).start();
    
    try {
      const { stdout } = await execa('finch', args);
      const imageId = this.extractImageId(stdout);
      spinner.succeed(`Built image ${tag} successfully`);
      return imageId;
    } catch (error) {
      spinner.fail(`Failed to build image ${tag}`);
      throw error;
    }
  }

  /**
   * Runs a container using Finch
   * @param options Run options
   * @returns Promise resolving to the container ID
   */
  async runContainer(options: FinchRunOptions): Promise<string> {
    if (!await this.ensureVmRunning()) {
      throw new Error('Finch VM is not running');
    }
    
    const { 
      imageName, 
      containerName, 
      ports = [], 
      envs = [], 
      volumes = [],
      detach = false,
      remove = false,
      network
    } = options;
    
    const args = ['run'];
    
    // Add optional arguments
    if (containerName) {
      args.push('--name', containerName);
    }
    
    if (detach) {
      args.push('-d');
    }
    
    if (remove) {
      args.push('--rm');
    }
    
    if (network) {
      args.push('--network', network);
    }
    
    if (options.interactive) {
      args.push('-i');
    }
    
    // Add ports
    ports.forEach(port => {
      args.push('-p', port);
    });
    
    // Add environment variables
    envs.forEach(env => {
      args.push('-e', env);
    });
    
    // Add volumes
    volumes.forEach(volume => {
      args.push('-v', volume);
    });
    
    // Add image name
    args.push(imageName);
    
    const spinner = ora(`Starting container from image ${imageName}...`).start();
    
    try {
      const { stdout } = await execa('finch', args);
      spinner.succeed(`Container started successfully`);
      return stdout.trim(); // Container ID
    } catch (error) {
      spinner.fail(`Failed to start container from image ${imageName}`);
      throw error;
    }
  }

  /**
   * Publishes a container image to a registry
   * @param options Publish options
   * @returns Promise resolving when the image is published
   */
  async publishImage(options: FinchPublishOptions): Promise<void> {
    if (!await this.ensureVmRunning()) {
      throw new Error('Finch VM is not running');
    }
    
    const { imageName, registry, tag } = options;
    
    // Tag the image for the registry
    const targetImage = `${registry}/${imageName}:${tag}`;
    await execa('finch', ['tag', `${imageName}:${tag}`, targetImage]);
    
    const spinner = ora(`Publishing image to ${targetImage}...`).start();
    
    try {
      await execa('finch', ['push', targetImage]);
      spinner.succeed(`Published image to ${targetImage} successfully`);
    } catch (error) {
      spinner.fail(`Failed to publish image to ${targetImage}`);
      throw error;
    }
  }

  /**
   * Extracts the image ID from the build output
   * @param output Build output
   * @returns Image ID
   */
  private extractImageId(output: string): string {
    // Example: "Successfully built 12345abcde"
    const match = output.match(/Successfully built ([a-f0-9]+)/);
    return match ? match[1] : '';
  }
}