"use strict";
/**
 * OpenAI Codex Integration Main Entry Point
 * This extension integrates spy-code with OpenAI Codex via agent templates
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
const codex_adapter_1 = require("@spy-code/codex-adapter");
const integration_core_1 = require("@spy-code/integration-core");
const fs = __importStar(require("fs/promises"));
const path = __importStar(require("path"));
let adapter;
let mcpClient;
let cacheManager;
async function activate(context) {
    console.log('Spy-Code OpenAI Codex integration is activating');
    // Initialize adapter
    adapter = new codex_adapter_1.CodexAdapter({
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
    // Setup Codex MCP configuration
    await setupCodexMCPConfig();
    // Register agent templates
    await registerAgentTemplates();
    console.log('Spy-Code OpenAI Codex integration activated successfully');
}
async function deactivate() {
    console.log('Spy-Code OpenAI Codex integration is deactivating');
    if (adapter) {
        await adapter.deactivate();
    }
    if (mcpClient) {
        await mcpClient.disconnect();
    }
    if (cacheManager) {
        await cacheManager.clear();
    }
    console.log('Spy-Code OpenAI Codex integration deactivated');
}
/**
 * Setup Codex MCP configuration
 */
async function setupCodexMCPConfig() {
    const codexDir = path.join(process.cwd(), '.codex');
    const mcpConfigPath = path.join(codexDir, 'mcp.json');
    try {
        await fs.mkdir(codexDir, { recursive: true });
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
        console.log('Codex MCP configuration written to', mcpConfigPath);
    }
    catch (error) {
        console.error('Failed to setup Codex MCP config:', error);
    }
}
/**
 * Register Codex agent templates
 */
async function registerAgentTemplates() {
    const agentsDir = path.join(process.cwd(), '.codex', 'agents');
    try {
        await fs.mkdir(agentsDir, { recursive: true });
        // Search agent template
        const searchAgent = {
            id: 'spy-code-search-agent',
            name: 'Spy-Code Search Agent',
            description: 'Agent for searching codebase using spy-code',
            tools: ['spy-code-search', 'spy-code-semantic-search'],
            capabilities: {
                search: true,
                navigation: true
            },
            instructions: `
You are a code search agent powered by spy-code.
Use the spy-code-search tool for keyword-based search.
Use the spy-code-semantic-search tool for natural language queries.
Provide context about the code you find.
      `.trim()
        };
        await fs.writeFile(path.join(agentsDir, 'search-agent.json'), JSON.stringify(searchAgent, null, 2));
        // Analysis agent template
        const analysisAgent = {
            id: 'spy-code-analysis-agent',
            name: 'Spy-Code Analysis Agent',
            description: 'Agent for code analysis using spy-code',
            tools: ['spy-code-get-node', 'spy-code-callers', 'spy-code-callees'],
            capabilities: {
                analysis: true,
                impact_analysis: true
            },
            instructions: `
You are a code analysis agent powered by spy-code.
Use spy-code-get-node to get detailed information about functions/classes.
Use spy-code-callers to find what calls a function.
Use spy-code-callees to find what a function calls.
Provide comprehensive analysis of code relationships.
      `.trim()
        };
        await fs.writeFile(path.join(agentsDir, 'analysis-agent.json'), JSON.stringify(analysisAgent, null, 2));
        // Review agent template
        const reviewAgent = {
            id: 'spy-code-review-agent',
            name: 'Spy-Code Review Agent',
            description: 'Agent for code review using spy-code',
            tools: ['spy-code-search', 'spy-code-semantic-search', 'spy-code-get-node'],
            capabilities: {
                review: true,
                pattern_detection: true
            },
            instructions: `
You are a code review agent powered by spy-code.
Use spy-code-search to find similar code patterns.
Use spy-code-semantic-search to find related code.
Use spy-code-get-node to understand function details.
Identify potential issues, duplicates, and improvements.
      `.trim()
        };
        await fs.writeFile(path.join(agentsDir, 'review-agent.json'), JSON.stringify(reviewAgent, null, 2));
        console.log('Codex agent templates registered');
    }
    catch (error) {
        console.error('Failed to register agent templates:', error);
    }
}
