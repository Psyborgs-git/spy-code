"use strict";
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
const SidebarProvider_1 = require("./SidebarProvider");
const McpServerManager_1 = require("./McpServerManager");
const commands_1 = require("./commands");
const providers_1 = require("./providers");
let adapter;
let cliBridge;
let mcpClient;
let cacheManager;
async function activate(context) {
    console.log('Spy-Code Unified extension is activating');
    const config = vscode.workspace.getConfiguration('spy-code');
    const adapterConfig = {
        spyCodePath: config.get('path', 'spy-code'),
        dbPath: config.get('dbPath', '.spy-code/graph.db'),
        enableMCP: true,
        enableHooks: true,
        cacheEnabled: true,
        cacheTTL: 300000,
        maxCacheSize: 1000
    };
    adapter = new vscode_adapter_1.VSCodeAdapter(context, adapterConfig);
    await adapter.initialize();
    cliBridge = new integration_core_1.CLIBridge({
        command: adapterConfig.spyCodePath,
        timeout: 30000
    });
    const mcpServerManager = new McpServerManager_1.McpServerManager();
    const sidebarProvider = new SidebarProvider_1.SidebarProvider(context.extensionUri, mcpServerManager, cliBridge);
    context.subscriptions.push(vscode.window.registerWebviewViewProvider('spy-code.sidebar', sidebarProvider));
    // Initialize MCP client if needed for native features
    try {
        mcpClient = new integration_core_1.MCPClient();
        await mcpClient.connect();
        await mcpClient.initialize();
    }
    catch (error) {
        console.warn('Failed to initialize MCP client', error);
    }
    cacheManager = new integration_core_1.CacheManager({
        maxCacheSize: adapterConfig.maxCacheSize,
        defaultTTL: adapterConfig.cacheTTL,
        diskCacheEnabled: true
    });
    (0, commands_1.registerCommands)(context, adapter, cliBridge, mcpClient, cacheManager, sidebarProvider);
    (0, providers_1.registerProviders)(context, cliBridge, mcpClient);
    context.subscriptions.push(vscode.commands.registerCommand('spy-code.openDashboard', () => {
        vscode.commands.executeCommand('spy-code.sidebar.focus');
    }));
    context.subscriptions.push(vscode.commands.registerCommand('spy-code.startMcpServer', async () => {
        await mcpServerManager.start();
        vscode.window.showInformationMessage('Spy-Code MCP Server Started');
    }));
    context.subscriptions.push(vscode.commands.registerCommand('spy-code.stopMcpServer', async () => {
        await mcpServerManager.stop();
        vscode.window.showInformationMessage('Spy-Code MCP Server Stopped');
    }));
}
async function deactivate() {
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
//# sourceMappingURL=extension.js.map