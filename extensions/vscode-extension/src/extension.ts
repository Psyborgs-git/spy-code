/**
 * VS Code Extension Main Entry Point
 */

import * as vscode from 'vscode';
import { VSCodeAdapter } from '@spy-code/vscode-adapter';
import { CLIBridge, MCPClient, CacheManager, eventBus, EventType } from '@spy-code/integration-core';
import { SidebarProvider } from './sidebar/sidebarProvider';
import { registerCommands } from './commands';
import { registerProviders } from './providers';

let adapter: VSCodeAdapter;
let cliBridge: CLIBridge;
let mcpClient: MCPClient;
let cacheManager: CacheManager;
let sidebarProvider: SidebarProvider;

export async function activate(context: vscode.ExtensionContext) {
  console.log('Spy-Code extension is activating');

  // Get configuration
  const config = vscode.workspace.getConfiguration('spy-code');
  const adapterConfig = {
    spyCodePath: config.get<string>('path', 'spy-code'),
    dbPath: config.get<string>('dbPath', '.spy-code/graph.db'),
    enableMCP: config.get<boolean>('enableMCP', true),
    enableHooks: config.get<boolean>('enableHooks', true),
    cacheEnabled: config.get<boolean>('cacheEnabled', true),
    cacheTTL: config.get<number>('cacheTTL', 300000),
    maxCacheSize: 1000
  };

  // Initialize adapter
  adapter = new VSCodeAdapter(context, adapterConfig);
  await adapter.initialize();

  // Initialize CLI bridge
  cliBridge = new CLIBridge({
    command: adapterConfig.spyCodePath,
    timeout: 30000
  });

  // Check if spy-code is available
  const isAvailable = await cliBridge.isAvailable();
  if (!isAvailable) {
    vscode.window.showWarningMessage(
      'Spy-Code CLI not found. Please install spy-code or configure the path in settings.'
    );
  }

  // Initialize MCP client if enabled
  if (adapterConfig.enableMCP && isAvailable) {
    try {
      mcpClient = new MCPClient();
      await mcpClient.connect();
      await mcpClient.initialize();
      eventBus.emit(EventType.MCP_CONNECTED, {});
    } catch (error) {
      console.error('Failed to initialize MCP client:', error);
      vscode.window.showWarningMessage('Failed to initialize MCP server');
    }
  }

  // Initialize cache manager
  if (adapterConfig.cacheEnabled) {
    cacheManager = new CacheManager({
      maxCacheSize: adapterConfig.maxCacheSize,
      defaultTTL: adapterConfig.cacheTTL,
      diskCacheEnabled: true
    });
  }

  // Initialize sidebar provider
  sidebarProvider = new SidebarProvider(
    context.extensionUri,
    cliBridge,
    mcpClient,
    cacheManager
  );

  // Register sidebar
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      'spy-code.sidebar',
      sidebarProvider
    )
  );

  // Register commands
  registerCommands(context, adapter, cliBridge, mcpClient, cacheManager, sidebarProvider);

  // Register language providers
  registerProviders(context, cliBridge, mcpClient);

  // Register configuration change listener
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration('spy-code')) {
        const newConfig = vscode.workspace.getConfiguration('spy-code');
        adapter.updateConfig({
          spyCodePath: newConfig.get<string>('path', 'spy-code'),
          dbPath: newConfig.get<string>('dbPath', '.spy-code/graph.db'),
          enableMCP: newConfig.get<boolean>('enableMCP', true),
          enableHooks: newConfig.get<boolean>('enableHooks', true),
          cacheEnabled: newConfig.get<boolean>('cacheEnabled', true),
          cacheTTL: newConfig.get<number>('cacheTTL', 300000)
        });
        
        // Reinitialize CLI bridge with new config
        cliBridge.updateConfig({
          command: newConfig.get<string>('path', 'spy-code')
        });
      }
    })
  );

  console.log('Spy-Code extension activated successfully');
}

export async function deactivate() {
  console.log('Spy-Code extension is deactivating');

  if (adapter) {
    await adapter.deactivate();
  }

  if (mcpClient) {
    await mcpClient.disconnect();
  }

  if (cacheManager) {
    await cacheManager.clear();
  }

  console.log('Spy-Code extension deactivated');
}
