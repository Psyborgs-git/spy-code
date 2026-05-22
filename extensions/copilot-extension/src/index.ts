/**
 * GitHub Copilot Extension Main Entry Point
 * This extension integrates spy-code with GitHub Copilot via MCP
 */

import { CopilotAdapter } from '@spy-code/copilot-adapter';
import { MCPClient, CacheManager, eventBus, EventType } from '@spy-code/integration-core';
import * as fs from 'fs/promises';
import * as path from 'path';

let adapter: CopilotAdapter;
let mcpClient: MCPClient;
let cacheManager: CacheManager;

export async function activate(context: any): Promise<void> {
  console.log('Spy-Code GitHub Copilot extension is activating');

  // Initialize adapter
  adapter = new CopilotAdapter({
    spyCodePath: 'spy-code',
    dbPath: '.spy-code/graph.db',
    enableMCP: true,
    enableHooks: true,
    cacheEnabled: true,
    cacheTTL: 300000,
    maxCacheSize: 1000
  });

  await adapter.initialize();
  await adapter.activate();

  // Initialize MCP client
  try {
    mcpClient = new MCPClient();
    await mcpClient.connect();
    await mcpClient.initialize();
    eventBus.emit(EventType.MCP_CONNECTED, {});
    console.log('MCP client connected');
  } catch (error) {
    console.error('Failed to initialize MCP client:', error);
  }

  // Initialize cache manager
  cacheManager = new CacheManager({
    maxCacheSize: 1000,
    defaultTTL: 300000,
    diskCacheEnabled: true
  });

  // Setup Copilot MCP configuration
  await setupCopilotMCPConfig();

  console.log('Spy-Code GitHub Copilot extension activated successfully');
}

export async function deactivate(): Promise<void> {
  console.log('Spy-Code GitHub Copilot extension is deactivating');

  if (adapter) {
    await adapter.deactivate();
  }

  if (mcpClient) {
    await mcpClient.disconnect();
  }

  if (cacheManager) {
    await cacheManager.clear();
  }

  console.log('Spy-Code GitHub Copilot extension deactivated');
}

/**
 * Setup Copilot MCP configuration
 */
async function setupCopilotMCPConfig(): Promise<void> {
  const copilotDir = path.join(process.cwd(), '.github', 'copilot');
  const mcpConfigPath = path.join(copilotDir, 'mcp.json');

  try {
    await fs.mkdir(copilotDir, { recursive: true });

    const mcpConfig = {
      mcpServers: {
        'spy-code': {
          command: 'spy-code',
          args: ['serve', '--mcp'],
          env: {
            SPY_DB_PATH: '.spy-code/graph.db'
          }
        }
      }
    };

    await fs.writeFile(mcpConfigPath, JSON.stringify(mcpConfig, null, 2));
    console.log('Copilot MCP configuration written to', mcpConfigPath);
  } catch (error) {
    console.error('Failed to setup Copilot MCP config:', error);
  }
}
