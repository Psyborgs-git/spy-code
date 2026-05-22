"use strict";
/**
 * Windsurf Extension Main Entry Point
 * Reuses VS Code extension code since Windsurf is VS Code-based
 * Includes integration with Windsurf's Cascade hooks
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const vscode_adapter_1 = require("@spy-code/vscode-adapter");
const integration_core_1 = require("@spy-code/integration-core");
const sidebarProvider_1 = require("./sidebar/sidebarProvider");
const commands_1 = require("./commands");
const providers_1 = require("./providers");
let adapter;
let cliBridge;
let mcpClient;
let cacheManager;
let sidebarProvider;
let agentHooks = (0, integration_core_1.getAgentHooks)();
async function activate(context) {
    console.log('Spy-Code Windsurf extension is activating');
    // Get configuration
    const config = vscode.workspace.getConfiguration('spy-code');
    const adapterConfig = {
        spyCodePath: config.get('path', 'spy-code'),
        dbPath: config.get('dbPath', '.spy-code/graph.db'),
        enableMCP: config.get('enableMCP', true),
        enableHooks: config.get('enableHooks', true),
        cacheEnabled: config.get('cacheEnabled', true),
        cacheTTL: config.get('cacheTTL', 300000),
        maxCacheSize: 1000
    };
    // Initialize adapter
    adapter = new vscode_adapter_1.VSCodeAdapter(context, adapterConfig);
    await adapter.initialize();
    // Initialize CLI bridge
    cliBridge = new integration_core_1.CLIBridge({
        command: adapterConfig.spyCodePath,
        timeout: 30000
    });
    // Check if spy-code is available
    const isAvailable = await cliBridge.isAvailable();
    if (!isAvailable) {
        vscode.window.showWarningMessage('Spy-Code CLI not found. Please install spy-code or configure the path in settings.');
    }
    // Initialize MCP client if enabled
    if (adapterConfig.enableMCP && isAvailable) {
        try {
            mcpClient = new integration_core_1.MCPClient();
            await mcpClient.connect();
            await mcpClient.initialize();
            integration_core_1.eventBus.emit(integration_core_1.EventType.MCP_CONNECTED, {});
        }
        catch (error) {
            console.error('Failed to initialize MCP client:', error);
            vscode.window.showWarningMessage('Failed to initialize MCP server');
        }
    }
    // Initialize cache manager
    if (adapterConfig.cacheEnabled) {
        cacheManager = new integration_core_1.CacheManager({
            maxCacheSize: adapterConfig.maxCacheSize,
            defaultTTL: adapterConfig.cacheTTL,
            diskCacheEnabled: true
        });
    }
    // Initialize sidebar provider
    sidebarProvider = new sidebarProvider_1.SidebarProvider(context.extensionUri, cliBridge, mcpClient, cacheManager);
    // Register sidebar
    context.subscriptions.push(vscode.window.registerWebviewViewProvider('spy-code.sidebar', sidebarProvider));
    // Register commands
    (0, commands_1.registerCommands)(context, adapter, cliBridge, mcpClient, cacheManager, sidebarProvider);
    // Register language providers
    (0, providers_1.registerProviders)(context, cliBridge, mcpClient);
    // Register Windsurf-specific Cascade hooks if enabled
    if (adapterConfig.enableHooks) {
        registerWindsurfHooks(context);
    }
    // Register configuration change listener
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration(async (e) => {
        if (e.affectsConfiguration('spy-code')) {
            const newConfig = vscode.workspace.getConfiguration('spy-code');
            adapter.updateConfig({
                spyCodePath: newConfig.get('path', 'spy-code'),
                dbPath: newConfig.get('dbPath', '.spy-code/graph.db'),
                enableMCP: newConfig.get('enableMCP', true),
                enableHooks: newConfig.get('enableHooks', true),
                cacheEnabled: newConfig.get('cacheEnabled', true),
                cacheTTL: newConfig.get('cacheTTL', 300000)
            });
            cliBridge.updateConfig({
                command: newConfig.get('path', 'spy-code')
            });
        }
    }));
    console.log('Spy-Code Windsurf extension activated successfully');
}
async function deactivate() {
    console.log('Spy-Code Windsurf extension is deactivating');
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
    console.log('Spy-Code Windsurf extension deactivated');
}
/**
 * Register Windsurf Cascade hooks
 */
function registerWindsurfHooks(context) {
    // Pre-read code hook - Add spy-code context before reading files
    agentHooks.registerHook(integration_core_1.HookType.PRE_READ_CODE, async (hookContext) => {
        try {
            const node = await cliBridge.getNode(hookContext.filePath);
            if (node) {
                hookContext.node = node;
            }
            return { continue: true };
        }
        catch (error) {
            return { continue: true };
        }
    });
    // Post-write code hook - Validate code changes using spy-code
    agentHooks.registerHook(integration_core_1.HookType.POST_WRITE_CODE, async (hookContext) => {
        try {
            // Reindex the changed file
            await cliBridge.reindex(false);
            return { continue: true };
        }
        catch (error) {
            console.error('Post-write hook error:', error);
            return { continue: true };
        }
    });
    // Pre-MCP tool use hook - Add spy-code context to MCP calls
    agentHooks.registerHook(integration_core_1.HookType.PRE_MCP_TOOL_USE, async (hookContext) => {
        try {
            const stats = await cliBridge.getStats();
            hookContext.agentState.contextSize = stats.nodeCount;
            return { continue: true };
        }
        catch (error) {
            return { continue: true };
        }
    });
    // Post-cascade response hook - Log spy-code usage
    agentHooks.registerHook(integration_core_1.HookType.POST_CASCADE_RESPONSE, async (hookContext) => {
        try {
            const stats = await cliBridge.getStats();
            console.log(`Spy-Code stats after Cascade response: ${stats.nodeCount} nodes indexed`);
            return { continue: true };
        }
        catch (error) {
            return { continue: true };
        }
    });
    console.log('Windsurf Cascade hooks registered');
}
//# sourceMappingURL=extension.js.map