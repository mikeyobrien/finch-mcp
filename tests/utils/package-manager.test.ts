import { describe, expect, test, beforeEach, afterEach, jest } from '@jest/globals';
import fs from 'fs-extra';
import path from 'path';
import os from 'os';
import { PackageManager, detectPackageManager, isPackageManagerInstalled, getInstallCommand, getRunCommand } from '../../src/utils/package-manager';

// Mock dependencies
jest.mock('execa');
import { execa } from 'execa';
const mockedExeca = jest.mocked(execa);

describe('Package Manager Utils', () => {
  let tempDir: string;

  beforeEach(async () => {
    // Create a temporary directory for tests
    tempDir = path.join(os.tmpdir(), `finch-mcp-test-${Date.now()}`);
    await fs.mkdir(tempDir, { recursive: true });
  });

  afterEach(async () => {
    // Clean up the temporary directory
    await fs.remove(tempDir);
  });

  describe('detectPackageManager', () => {
    test('should return null if no package.json exists', async () => {
      const result = await detectPackageManager(tempDir);
      expect(result).toBeNull();
    });

    test('should detect npm as package manager', async () => {
      // Create a package.json file and package-lock.json
      await fs.writeJSON(path.join(tempDir, 'package.json'), {
        name: 'test-project',
        dependencies: {
          express: '^4.17.1'
        }
      });
      await fs.writeFile(path.join(tempDir, 'package-lock.json'), '{}');
      
      // Mock npm --version
      mockedExeca.mockResolvedValueOnce({ stdout: '8.1.2', stderr: '' } as any);

      const result = await detectPackageManager(tempDir);
      
      expect(result).not.toBeNull();
      expect(result?.type).toBe(PackageManager.NPM);
      expect(result?.lockfileExists).toBe(true);
      expect(result?.version).toBe('8.1.2');
    });

    test('should detect uv as package manager', async () => {
      // Create a package.json file and uv-lock.json
      await fs.writeJSON(path.join(tempDir, 'package.json'), {
        name: 'test-project',
        dependencies: {
          express: '^4.17.1'
        }
      });
      await fs.writeFile(path.join(tempDir, 'uv-lock.json'), '{}');
      
      // Mock uv --version
      mockedExeca.mockResolvedValueOnce({ stdout: '0.1.0', stderr: '' } as any);

      const result = await detectPackageManager(tempDir);
      
      expect(result).not.toBeNull();
      expect(result?.type).toBe(PackageManager.UV);
      expect(result?.lockfileExists).toBe(true);
      expect(result?.version).toBe('0.1.0');
    });
  });

  describe('isPackageManagerInstalled', () => {
    test('should return true if package manager is installed', async () => {
      mockedExeca.mockResolvedValueOnce({ stdout: '8.1.2', stderr: '' } as any);
      
      const result = await isPackageManagerInstalled(PackageManager.NPM);
      
      expect(result).toBe(true);
    });

    test('should return false if package manager is not installed', async () => {
      mockedExeca.mockRejectedValueOnce(new Error('Command not found'));
      
      const result = await isPackageManagerInstalled(PackageManager.UV);
      
      expect(result).toBe(false);
    });
  });

  describe('getInstallCommand', () => {
    test('should return npm ci for NPM', () => {
      const result = getInstallCommand(PackageManager.NPM);
      expect(result).toBe('npm ci');
    });

    test('should return uv install for UV', () => {
      const result = getInstallCommand(PackageManager.UV);
      expect(result).toBe('uv install');
    });
  });

  describe('getRunCommand', () => {
    test('should return npm run for NPM', () => {
      const result = getRunCommand(PackageManager.NPM, 'start');
      expect(result).toBe('npm run start');
    });

    test('should return uv run for UV', () => {
      const result = getRunCommand(PackageManager.UV, 'start');
      expect(result).toBe('uv run start');
    });
  });
});