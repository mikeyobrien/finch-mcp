import fs from 'fs-extra';
import path from 'path';
import { execa } from 'execa';

/**
 * Supported package managers
 */
export enum PackageManager {
  NPM = 'npm',
  UV = 'uv'
}

/**
 * Package manager detection result
 */
export interface PackageManagerInfo {
  type: PackageManager;
  version: string;
  lockfileExists: boolean;
  packageJson?: Record<string, any>;
}

/**
 * Detects the package manager used in a project
 * @param projectPath Path to the project directory
 * @returns Information about the detected package manager
 */
export async function detectPackageManager(projectPath: string): Promise<PackageManagerInfo | null> {
  try {
    // Check if package.json exists
    const packageJsonPath = path.join(projectPath, 'package.json');
    if (!await fs.pathExists(packageJsonPath)) {
      return null;
    }

    // Read package.json
    const packageJson = await fs.readJson(packageJsonPath);
    
    // Check for lock files to determine package manager
    const packageLockExists = await fs.pathExists(path.join(projectPath, 'package-lock.json'));
    const uvLockExists = await fs.pathExists(path.join(projectPath, 'uv-lock.json'));
    
    // Determine package manager type
    let type: PackageManager;
    let version: string;
    
    if (uvLockExists) {
      type = PackageManager.UV;
      try {
        const { stdout } = await execa('uv', ['--version']);
        version = stdout.trim();
      } catch (error) {
        // If uv command fails, assume it's not installed
        version = 'unknown';
      }
    } else {
      type = PackageManager.NPM;
      try {
        const { stdout } = await execa('npm', ['--version']);
        version = stdout.trim();
      } catch (error) {
        // If npm command fails, assume it's not installed
        version = 'unknown';
      }
    }
    
    return {
      type,
      version,
      lockfileExists: type === PackageManager.NPM ? packageLockExists : uvLockExists,
      packageJson
    };
  } catch (error) {
    // If any error occurs during detection, return null
    console.error('Error detecting package manager:', error);
    return null;
  }
}

/**
 * Checks if a package manager is installed
 * @param manager Package manager to check
 * @returns true if the package manager is installed, false otherwise
 */
export async function isPackageManagerInstalled(manager: PackageManager): Promise<boolean> {
  try {
    await execa(manager, ['--version']);
    return true;
  } catch (error) {
    return false;
  }
}

/**
 * Gets the install command for a package manager
 * @param manager Package manager to get the install command for
 * @returns The install command
 */
export function getInstallCommand(manager: PackageManager): string {
  switch (manager) {
    case PackageManager.NPM:
      return 'npm ci';
    case PackageManager.UV:
      return 'uv install';
    default:
      return 'npm install';
  }
}

/**
 * Gets the run command for a package manager
 * @param manager Package manager to get the run command for
 * @param script Script to run
 * @returns The run command
 */
export function getRunCommand(manager: PackageManager, script: string): string {
  switch (manager) {
    case PackageManager.NPM:
      return `npm run ${script}`;
    case PackageManager.UV:
      return `uv run ${script}`;
    default:
      return `npm run ${script}`;
  }
}