/**
 * Environment Detection Tests
 * Tests for environment detection utility
 */

import { detectEnvironment, getConfigPath, getMcpConfigTemplate } from '../../scripts/detect-environment';

describe('Environment Detection', () => {
  it('should detect environment', () => {
    const env = detectEnvironment();
    expect(env).toBeDefined();
    expect(['cursor', 'windsurf', 'claude', 'vscode', 'generic']).toContain(env);
  });
  
  it('should return config path for detected environment', () => {
    const env = detectEnvironment();
    const configPath = getConfigPath(env);
    
    if (env !== 'generic') {
      expect(configPath).toBeDefined();
      expect(typeof configPath).toBe('string');
    } else {
      expect(configPath).toBeNull();
    }
  });
  
  it('should return config template for environment', () => {
    const env = detectEnvironment();
    const template = getMcpConfigTemplate(env);
    
    if (env !== 'generic') {
      expect(template).toBeDefined();
      expect(typeof template).toBe('string');
      expect(template).toContain('mcpServers');
    } else {
      expect(template).toBeNull();
    }
  });
  
  it('should handle all environment types', () => {
    const environments = ['cursor', 'windsurf', 'claude', 'vscode', 'generic'];
    
    for (const env of environments) {
      const configPath = getConfigPath(env);
      if (env !== 'generic') {
        expect(configPath).toBeDefined();
      }
    }
  });
});
