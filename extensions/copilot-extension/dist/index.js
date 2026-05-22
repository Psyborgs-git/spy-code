"use strict";
/**
 * GitHub Copilot Extension Main Entry Point
 * This extension integrates spy-code with GitHub Copilot via MCP
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
const copilot_adapter_1 = require("@spy-code/copilot-adapter");
const integration_core_1 = require("@spy-code/integration-core");
const fs = __importStar(require("fs/promises"));
const path = __importStar(require("path"));
let adapter;
let mcpClient;
let cacheManager;
async function activate(context) {
    console.log('Spy-Code GitHub Copilot extension is activating');
    // Initialize adapter
    adapter = new copilot_adapter_1.CopilotAdapter({
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
        mcpClient = new integration_core_1.MCPClient();
        await mcpClient.connect();
        await mcpClient.initialize();
        integration_core_1.eventBus.emit(integration_core_1.EventType.MCP_CONNECTED, {});
        console.log('MCP client connected');
    }
    catch (error) {
        console.error('Failed to initialize MCP client:', error);
    }
    // Initialize cache manager
    cacheManager = new integration_core_1.CacheManager({
        maxCacheSize: 1000,
        defaultTTL: 300000,
        diskCacheEnabled: true
    });
    // Setup Copilot MCP configuration
    await setupCopilotMCPConfig();
    console.log('Spy-Code GitHub Copilot extension activated successfully');
}
async function deactivate() {
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
async function setupCopilotMCPConfig() {
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
    }
    catch (error) {
        console.error('Failed to setup Copilot MCP config:', error);
    }
}
