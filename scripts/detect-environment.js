#!/usr/bin/env node

/**
 * Environment Detection Utility
 * Detects the current AI coding environment
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

function detectEnvironment() {
  const homeDir = os.homedir();
  
  // Check for Cursor
  if (fs.existsSync(path.join(homeDir, '.cursor')) || 
      fs.existsSync(path.join(homeDir, '.cursor', 'mcp_config.json'))) {
    return 'cursor';
  }
  
  // Check for Windsurf/Cascade
  if (fs.existsSync(path.join(homeDir, '.windsurf')) || 
      fs.existsSync(path.join(homeDir, '.windsurf', 'mcp_config.json'))) {
    return 'windsurf';
  }
  
  // Check for Claude Desktop
  const claudeConfigPaths = [
    path.join(homeDir, 'Library', 'Application Support', 'Claude', 'claude_desktop_config.json'),
    path.join(homeDir, '.config', 'Claude', 'claude_desktop_config.json')
  ];
  
  if (claudeConfigPaths.some(p => fs.existsSync(p))) {
    return 'claude';
  }
  
  // Check for VS Code
  if (fs.existsSync(path.join(homeDir, '.vscode')) || 
      fs.existsSync(path.join(homeDir, '.vscode-insiders'))) {
    return 'vscode';
  }
  
  return 'generic';
}

function getConfigPath(environment) {
  const homeDir = os.homedir();
  
  const configPaths = {
    cursor: path.join(homeDir, '.cursor', 'mcp_config.json'),
    windsurf: path.join(homeDir, '.windsurf', 'mcp_config.json'),
    claude: path.join(homeDir, 'Library', 'Application Support', 'Claude', 'claude_desktop_config.json'),
    vscode: path.join(homeDir, '.vscode', 'mcp_config.json'),
    generic: null
  };
  
  // Fallback for Linux Claude config
  if (environment === 'claude' && !fs.existsSync(configPaths.claude)) {
    configPaths.claude = path.join(homeDir, '.config', 'Claude', 'claude_desktop_config.json');
  }
  
  return configPaths[environment] || null;
}

function getMcpConfigTemplate(environment) {
  const repoRoot = process.cwd();
  const templatePath = path.join(repoRoot, 'configs', 'mcp', `${environment}.json`);
  
  if (fs.existsSync(templatePath)) {
    return fs.readFileSync(templatePath, 'utf8');
  }
  
  return null;
}

function mergeMcpConfig(existingConfig, newConfig) {
  try {
    const existing = JSON.parse(existingConfig);
    const newSpyConfig = JSON.parse(newConfig);
    
    if (!existing.mcpServers) {
      existing.mcpServers = {};
    }
    
    // Merge spy-code server config
    Object.assign(existing.mcpServers, newSpyConfig.mcpServers);
    
    return JSON.stringify(existing, null, 2);
  } catch (error) {
    console.error('Error merging configs:', error);
    return existingConfig;
  }
}

// CLI interface
if (require.main === module) {
  const args = process.argv.slice(2);
  const command = args[0];
  
  switch (command) {
    case 'detect':
      console.log(detectEnvironment());
      break;
    case 'config-path':
      console.log(getConfigPath(detectEnvironment()) || 'null');
      break;
    case 'template':
      const env = args[1] || detectEnvironment();
      console.log(getMcpConfigTemplate(env) || 'null');
      break;
    default:
      console.log('Usage: node detect-environment.js [detect|config-path|template] [env]');
      console.log('Commands:');
      console.log('  detect       - Detect current environment');
      console.log('  config-path  - Get MCP config path for detected environment');
      console.log('  template     - Get MCP config template for environment');
  }
}

module.exports = {
  detectEnvironment,
  getConfigPath,
  getMcpConfigTemplate,
  mergeMcpConfig
};
