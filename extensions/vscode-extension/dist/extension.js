"use strict";
/**
 * VS Code Extension Main Entry Point
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
async function activate(context) {
    console.log('Spy-Code extension is activating');
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
            // Reinitialize CLI bridge with new config
            cliBridge.updateConfig({
                command: newConfig.get('path', 'spy-code')
            });
        }
    }));
    console.log('Spy-Code extension activated successfully');
}
async function deactivate() {
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
//# sourceMappingURL=extension.js.map