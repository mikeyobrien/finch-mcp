import { describe, expect, test, beforeEach, afterEach, jest } from '@jest/globals';
import { FinchClient, FinchBuildOptions, FinchRunOptions, FinchPublishOptions } from '../../src/finch/finch-client';

// Mock dependencies
jest.mock('execa');
import { execa } from 'execa';
const mockedExeca = jest.mocked(execa);

// Mock ora for spinners
jest.mock('ora', () => {
  return jest.fn().mockImplementation(() => ({
    start: jest.fn().mockReturnThis(),
    succeed: jest.fn().mockReturnThis(),
    fail: jest.fn().mockReturnThis(),
    stop: jest.fn().mockReturnThis(),
    text: ''
  }));
});

describe('FinchClient', () => {
  let finchClient: FinchClient;

  beforeEach(() => {
    // Create an instance of FinchClient
    finchClient = new FinchClient();
    
    // Reset all mocks
    jest.resetAllMocks();
  });

  describe('isFinchAvailable', () => {
    test('should return true if Finch is installed', async () => {
      mockedExeca.mockResolvedValueOnce({ stdout: 'finch version 0.1.0', stderr: '' } as any);
      
      const result = await finchClient.isFinchAvailable();
      
      expect(result).toBe(true);
      expect(mockedExeca).toHaveBeenCalledWith('finch', ['version']);
    });

    test('should return false if Finch is not installed', async () => {
      mockedExeca.mockRejectedValueOnce(new Error('Command not found'));
      
      const result = await finchClient.isFinchAvailable();
      
      expect(result).toBe(false);
      expect(mockedExeca).toHaveBeenCalledWith('finch', ['version']);
    });
  });

  describe('ensureVmRunning', () => {
    test('should return true if VM is already running', async () => {
      mockedExeca.mockResolvedValueOnce({ stdout: 'running', stderr: '' } as any);
      
      const result = await finchClient.ensureVmRunning();
      
      expect(result).toBe(true);
      expect(mockedExeca).toHaveBeenCalledWith('finch', ['vm', 'status']);
    });

    test('should start VM if not running', async () => {
      mockedExeca.mockResolvedValueOnce({ stdout: 'stopped', stderr: '' } as any);
      mockedExeca.mockResolvedValueOnce({ stdout: '', stderr: '' } as any);
      
      const result = await finchClient.ensureVmRunning();
      
      expect(result).toBe(true);
      expect(mockedExeca).toHaveBeenCalledWith('finch', ['vm', 'status']);
      expect(mockedExeca).toHaveBeenCalledWith('finch', ['vm', 'start']);
    });

    test('should return false if VM fails to start', async () => {
      mockedExeca.mockResolvedValueOnce({ stdout: 'stopped', stderr: '' } as any);
      mockedExeca.mockRejectedValueOnce(new Error('Failed to start VM'));
      
      const result = await finchClient.ensureVmRunning();
      
      expect(result).toBe(false);
      expect(mockedExeca).toHaveBeenCalledWith('finch', ['vm', 'status']);
      expect(mockedExeca).toHaveBeenCalledWith('finch', ['vm', 'start']);
    });
  });

  describe('buildImage', () => {
    test('should build an image successfully', async () => {
      // Mock ensureVmRunning to return true
      jest.spyOn(finchClient, 'ensureVmRunning').mockResolvedValueOnce(true);
      
      // Mock execa to return successful build
      mockedExeca.mockResolvedValueOnce({ 
        stdout: 'Successfully built 12345abcde', 
        stderr: '' 
      } as any);
      
      const buildOptions: FinchBuildOptions = {
        tag: 'test-image:latest',
        contextDir: '/path/to/context'
      };
      
      const imageId = await finchClient.buildImage(buildOptions);
      
      expect(imageId).toBe('12345abcde');
      expect(mockedExeca).toHaveBeenCalledWith('finch', [
        'build', 
        '-t', 
        'test-image:latest', 
        '/path/to/context'
      ]);
    });

    test('should include all build options', async () => {
      // Mock ensureVmRunning to return true
      jest.spyOn(finchClient, 'ensureVmRunning').mockResolvedValueOnce(true);
      
      // Mock execa to return successful build
      mockedExeca.mockResolvedValueOnce({ 
        stdout: 'Successfully built 12345abcde', 
        stderr: '' 
      } as any);
      
      const buildOptions: FinchBuildOptions = {
        tag: 'test-image:latest',
        contextDir: '/path/to/context',
        dockerfilePath: '/path/to/Dockerfile',
        platform: 'linux/amd64',
        noCache: true,
        buildArgs: { VERSION: '1.0.0', ENV: 'production' }
      };
      
      const imageId = await finchClient.buildImage(buildOptions);
      
      expect(imageId).toBe('12345abcde');
      expect(mockedExeca).toHaveBeenCalledWith('finch', [
        'build', 
        '-t', 
        'test-image:latest',
        '-f',
        '/path/to/Dockerfile',
        '--platform',
        'linux/amd64',
        '--no-cache',
        '--build-arg',
        'VERSION=1.0.0',
        '--build-arg',
        'ENV=production',
        '/path/to/context'
      ]);
    });

    test('should throw error if build fails', async () => {
      // Mock ensureVmRunning to return true
      jest.spyOn(finchClient, 'ensureVmRunning').mockResolvedValueOnce(true);
      
      // Mock execa to return a build error
      mockedExeca.mockRejectedValueOnce(new Error('Build failed'));
      
      const buildOptions: FinchBuildOptions = {
        tag: 'test-image:latest',
        contextDir: '/path/to/context'
      };
      
      await expect(finchClient.buildImage(buildOptions)).rejects.toThrow('Build failed');
    });
  });

  describe('runContainer', () => {
    test('should run a container successfully', async () => {
      // Mock ensureVmRunning to return true
      jest.spyOn(finchClient, 'ensureVmRunning').mockResolvedValueOnce(true);
      
      // Mock execa to return successful container run
      mockedExeca.mockResolvedValueOnce({ 
        stdout: 'container-id-123', 
        stderr: '' 
      } as any);
      
      const runOptions: FinchRunOptions = {
        imageName: 'test-image:latest'
      };
      
      const containerId = await finchClient.runContainer(runOptions);
      
      expect(containerId).toBe('container-id-123');
      expect(mockedExeca).toHaveBeenCalledWith('finch', [
        'run',
        'test-image:latest'
      ]);
    });

    test('should include all run options', async () => {
      // Mock ensureVmRunning to return true
      jest.spyOn(finchClient, 'ensureVmRunning').mockResolvedValueOnce(true);
      
      // Mock execa to return successful container run
      mockedExeca.mockResolvedValueOnce({ 
        stdout: 'container-id-123', 
        stderr: '' 
      } as any);
      
      const runOptions: FinchRunOptions = {
        imageName: 'test-image:latest',
        containerName: 'test-container',
        ports: ['8080:80', '3000:3000'],
        envs: ['NODE_ENV=production', 'DEBUG=true'],
        volumes: ['/host/path:/container/path'],
        detach: true,
        remove: true,
        network: 'test-network'
      };
      
      const containerId = await finchClient.runContainer(runOptions);
      
      expect(containerId).toBe('container-id-123');
      expect(mockedExeca).toHaveBeenCalledWith('finch', [
        'run',
        '--name',
        'test-container',
        '-d',
        '--rm',
        '--network',
        'test-network',
        '-p',
        '8080:80',
        '-p',
        '3000:3000',
        '-e',
        'NODE_ENV=production',
        '-e',
        'DEBUG=true',
        '-v',
        '/host/path:/container/path',
        'test-image:latest'
      ]);
    });

    test('should throw error if container run fails', async () => {
      // Mock ensureVmRunning to return true
      jest.spyOn(finchClient, 'ensureVmRunning').mockResolvedValueOnce(true);
      
      // Mock execa to return a run error
      mockedExeca.mockRejectedValueOnce(new Error('Container run failed'));
      
      const runOptions: FinchRunOptions = {
        imageName: 'test-image:latest'
      };
      
      await expect(finchClient.runContainer(runOptions)).rejects.toThrow('Container run failed');
    });
  });

  describe('publishImage', () => {
    test('should publish an image successfully', async () => {
      // Mock ensureVmRunning to return true
      jest.spyOn(finchClient, 'ensureVmRunning').mockResolvedValueOnce(true);
      
      // Mock execa to return successful tag and push
      mockedExeca.mockResolvedValueOnce({ stdout: '', stderr: '' } as any); // tag
      mockedExeca.mockResolvedValueOnce({ stdout: '', stderr: '' } as any); // push
      
      const publishOptions: FinchPublishOptions = {
        imageName: 'test-image',
        registry: 'docker.io/username',
        tag: 'latest'
      };
      
      await finchClient.publishImage(publishOptions);
      
      expect(mockedExeca).toHaveBeenCalledWith('finch', [
        'tag',
        'test-image:latest',
        'docker.io/username/test-image:latest'
      ]);
      
      expect(mockedExeca).toHaveBeenCalledWith('finch', [
        'push',
        'docker.io/username/test-image:latest'
      ]);
    });

    test('should throw error if publish fails', async () => {
      // Mock ensureVmRunning to return true
      jest.spyOn(finchClient, 'ensureVmRunning').mockResolvedValueOnce(true);
      
      // Mock execa to return a successful tag but failed push
      mockedExeca.mockResolvedValueOnce({ stdout: '', stderr: '' } as any); // tag
      mockedExeca.mockRejectedValueOnce(new Error('Push failed')); // push
      
      const publishOptions: FinchPublishOptions = {
        imageName: 'test-image',
        registry: 'docker.io/username',
        tag: 'latest'
      };
      
      await expect(finchClient.publishImage(publishOptions)).rejects.toThrow('Push failed');
    });
  });
});