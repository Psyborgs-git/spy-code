/**
 * MCP Configuration Tests
 * Tests for MCP configuration templates
 */

import * as fs from 'fs';
import * as path from 'path';

describe('MCP Configurations', () => {
  const configsDir = path.join(__dirname, '../../configs/mcp');
  
  const environments = ['cursor', 'windsurf', 'claude-desktop', 'copilot', 'generic'];
  
  environments.forEach(env => {
    it(`should have valid JSON for ${env} config`, () => {
      const configPath = path.join(configsDir, `${env}.json`);
      const content = fs.readFileSync(configPath, 'utf8');
      
      expect(() => JSON.parse(content)).not.toThrow();
    });
    
    it(`should have mcpServers in ${env} config`, () => {
      const configPath = path.join(configsDir, `${env}.json`);
      const content = fs.readFileSync(configPath, 'utf8');
      const config = JSON.parse(content);
      
      expect(config).toHaveProperty('mcpServers');
      expect(config.mcpServers).toHaveProperty('spy-code');
    });
    
    it(`should have spy-code command in ${env} config`, () => {
      const configPath = path.join(configsDir, `${env}.json`);
      const content = fs.readFileSync(configPath, 'utf8');
      const config = JSON.parse(content);
      
      expect(config.mcpServers['spy-code']).toHaveProperty('command');
      expect(config.mcpServers['spy-code'].command).toBe('spy-code');
    });
    
    it(`should have serve --mcp args in ${env} config`, () => {
      const configPath = path.join(configsDir, `${env}.json`);
      const content = fs.readFileSync(configPath, 'utf8');
      const config = JSON.parse(content);
      
      expect(config.mcpServers['spy-code']).toHaveProperty('args');
      expect(config.mcpServers['spy-code'].args).toContain('serve');
      expect(config.mcpServers['spy-code'].args).toContain('--mcp');
    });
  });
  
  it('should have workspaceFolder environment variable', () => {
    const configPath = path.join(configsDir, 'cursor.json');
    const content = fs.readFileSync(configPath, 'utf8');
    const config = JSON.parse(content);
    
    expect(config.mcpServers['spy-code']).toHaveProperty('env');
    expect(config.mcpServers['spy-code'].env).toHaveProperty('SPY_CODE_CONFIG_PATH');
    expect(config.mcpServers['spy-code'].env.SPY_CODE_CONFIG_PATH).toContain('${workspaceFolder}');
  });
});
