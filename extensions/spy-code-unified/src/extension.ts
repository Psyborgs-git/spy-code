import * as vscode from 'vscode';
import { VSCodeAdapter } from '@spy-code/vscode-adapter';
import { CLIBridge, MCPClient, CacheManager } from '@spy-code/integration-core';
import { SidebarProvider } from './SidebarProvider';
import { McpServerManager } from './McpServerManager';
import { registerCommands } from './commands';
import { registerProviders } from './providers';

let adapter: VSCodeAdapter;
let cliBridge: CLIBridge;
let mcpClient: MCPClient;
let cacheManager: CacheManager;

export async function activate(context: vscode.ExtensionContext) {
  console.log('Spy-Code Unified extension is activating');

  const config = vscode.workspace.getConfiguration('spy-code');
  const adapterConfig = {
    spyCodePath: config.get<string>('path', 'spy-code'),
    dbPath: config.get<string>('dbPath', '.spy-code/graph.db'),
    enableMCP: true,
    enableHooks: true,
    cacheEnabled: true,
    cacheTTL: 300000,
    maxCacheSize: 1000
  };

  adapter = new VSCodeAdapter(context, adapterConfig);
  await adapter.initialize();

  cliBridge = new CLIBridge({
    command: adapterConfig.spyCodePath,
    timeout: 30000
  });

  mcpServerManager = new McpServerManager();
  const sidebarProvider = new SidebarProvider(context.extensionUri, mcpServerManager, cliBridge);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      'spy-code.sidebar',
      sidebarProvider
    )
  );

  // Initialize MCP client if needed for native features
  try {
      mcpClient = new MCPClient();
      await mcpClient.connect();
      await mcpClient.initialize();
  } catch (error) {
      console.warn('Failed to initialize MCP client', error);
  }

  cacheManager = new CacheManager({
      maxCacheSize: adapterConfig.maxCacheSize,
      defaultTTL: adapterConfig.cacheTTL,
      diskCacheEnabled: true
  });

  registerCommands(context, adapter, cliBridge, mcpClient, cacheManager, sidebarProvider as any);
  registerProviders(context, cliBridge, mcpClient);

  context.subscriptions.push(
    vscode.commands.registerCommand('spy-code.openDashboard', () => {
      vscode.commands.executeCommand('spy-code.sidebar.focus');
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('spy-code.startMcpServer', async () => {
      await mcpServerManager.start();
      vscode.window.showInformationMessage('Spy-Code MCP Server Started');
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('spy-code.stopMcpServer', async () => {
      await mcpServerManager.stop();
      vscode.window.showInformationMessage('Spy-Code MCP Server Stopped');
    })
  );
}

export async function deactivate() {
  console.log('Spy-Code Unified extension deactivated');
  if (adapter) {
    await adapter.deactivate();
  }
  if (mcpClient) {
    await mcpClient.disconnect();
  }
  if (cacheManager) {
    await cacheManager.clear();
  }
}
