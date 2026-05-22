#!/usr/bin/env node

/**
 * MCP Setup Script for npm package
 * Automatically configures MCP for the detected AI coding environment
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const { detectEnvironment, getConfigPath, getMcpConfigTemplate, mergeMcpConfig } = require('../../scripts/detect-environment');

function setupMcp() {
  console.log('Setting up spy-code MCP integration...');
  
  // Detect environment
  const env = detectEnvironment();
  console.log(`Detected environment: ${env}`);
  
  // Get config path
  const configPath = getConfigPath(env);
  if (!configPath) {
    console.log('No MCP config path for this environment. Manual setup required.');
    console.log('Config template available at: configs/mcp/generic.json');
    return;
  }
  
  console.log(`Config path: ${configPath}`);
  
  // Get template
  const template = getMcpConfigTemplate(env);
  if (!template) {
    console.error(`MCP config template not found for environment: ${env}`);
    return;
  }
  
  // Create config directory if needed
  const configDir = path.dirname(configPath);
  if (!fs.existsSync(configDir)) {
    console.log(`Creating config directory: ${configDir}`);
    fs.mkdirSync(configDir, { recursive: true });
  }
  
  // Backup existing config
  if (fs.existsSync(configPath)) {
    const backupPath = `${configPath}.backup`;
    console.log(`Backing up existing config to: ${backupPath}`);
    fs.copyFileSync(configPath, backupPath);
  }
  
  // Write or merge config
  if (fs.existsSync(configPath)) {
    console.log('Merging spy-code MCP configuration...');
    const existingConfig = fs.readFileSync(configPath, 'utf8');
    const mergedConfig = mergeMcpConfig(existingConfig, template);
    fs.writeFileSync(configPath, mergedConfig, 'utf8');
  } else {
    console.log('Creating new MCP configuration...');
    fs.writeFileSync(configPath, template, 'utf8');
  }
  
  console.log('✓ MCP configuration completed');
  console.log(`Config file: ${configPath}`);
  console.log('');
  console.log('Next steps:');
  console.log('1. Restart your AI coding environment');
  console.log('2. Test with: spy-code search "main"');
}

if (require.main === module) {
  setupMcp();
}

module.exports = { setupMcp };
