/**
 * Antigravity Extension Main Entry Point
 * Reuses VS Code extension code since Antigravity is VS Code-based
 * Includes integration with Antigravity's multi-agent workflow system
 */

import * as vscode from 'vscode';
import { VSCodeAdapter } from '@spy-code/vscode-adapter';
import { CLIBridge, MCPClient, CacheManager, eventBus, EventType, getAgentHooks, HookType } from '@spy-code/integration-core';
import { SidebarProvider } from './sidebar/sidebarProvider';
import { registerCommands } from './commands';
import { registerProviders } from './providers';

let adapter: VSCodeAdapter;
let cliBridge: CLIBridge;
let mcpClient: MCPClient;
let cacheManager: CacheManager;
let sidebarProvider: SidebarProvider;
let agentHooks = getAgentHooks();

export async function activate(context: vscode.ExtensionContext) {
  console.log('Spy-Code Antigravity extension is activating');

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

  // Register Antigravity-specific multi-agent hooks if enabled
  if (adapterConfig.enableHooks) {
    registerAntigravityHooks(context);
  }

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

        cliBridge.updateConfig({
          command: newConfig.get<string>('path', 'spy-code')
        });
      }
    })
  );

  console.log('Spy-Code Antigravity extension activated successfully');
}

export async function deactivate() {
  console.log('Spy-Code Antigravity extension is deactivating');

  if (adapter) {
    await adapter.deactivate();
  }

  if (mcpClient) {
    await mcpClient.disconnect();
  }

  if (cacheManager) {
    await cacheManager.clear();
  }

  if (agentHooks) {
    agentHooks.clearAll();
  }

  console.log('Spy-Code Antigravity extension deactivated');
}

/**
 * Register Antigravity multi-agent hooks
 */
function registerAntigravityHooks(context: vscode.ExtensionContext): void {
  // Pre-read code hook - Add spy-code context for multi-agent coordination
  agentHooks.registerHook(HookType.PRE_READ_CODE, async (hookContext) => {
    try {
      const node = await cliBridge.getNode(hookContext.filePath);
      if (node) {
        hookContext.node = node;
      }
      return { continue: true };
    } catch (error) {
      return { continue: true };
    }
  });

  // Post-write code hook - Validate and reindex after agent writes
  agentHooks.registerHook(HookType.POST_WRITE_CODE, async (hookContext) => {
    try {
      // Reindex the changed file
      await cliBridge.reindex(false);

      // Check for breaking changes
      const changedNodes = await cliBridge.changedSince('HEAD~1');
      if (changedNodes.length > 0) {
        console.log(`Detected ${changedNodes.length} changed nodes after write`);
      }

      return { continue: true };
    } catch (error) {
      console.error('Post-write hook error:', error);
      return { continue: true };
    }
  });

  // Pre-run command hook - Add spy-code context before agent commands
  agentHooks.registerHook(HookType.PRE_RUN_COMMAND, async (hookContext) => {
    try {
      const stats = await cliBridge.getStats();
      hookContext.agentState.contextSize = stats.nodeCount;
      return { continue: true };
    } catch (error) {
      return { continue: true };
    }
  });

  // Post-run command hook - Update index after agent commands
  agentHooks.registerHook(HookType.POST_RUN_COMMAND, async (hookContext) => {
    try {
      // Incremental reindex after command execution
      await cliBridge.reindex(false);
      return { continue: true };
    } catch (error) {
      console.error('Post-run command hook error:', error);
      return { continue: true };
    }
  });

  // Pre-MCP tool use hook - Enrich MCP calls with spy-code context
  agentHooks.registerHook(HookType.PRE_MCP_TOOL_USE, async (hookContext) => {
    try {
      const stats = await cliBridge.getStats();
      hookContext.agentState.contextSize = stats.nodeCount;
      return { continue: true };
    } catch (error) {
      return { continue: true };
    }
  });

  // Post-setup worktree hook - Index new worktree
  agentHooks.registerHook(HookType.POST_SETUP_WORKTREE, async (hookContext) => {
    try {
      // Full reindex for new worktree
      await cliBridge.reindex(true);
      return { continue: true };
    } catch (error) {
      console.error('Post-setup worktree hook error:', error);
      return { continue: true };
    }
  });

  console.log('Antigravity multi-agent hooks registered');
}
