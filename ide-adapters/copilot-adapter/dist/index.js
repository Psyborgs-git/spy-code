"use strict";
/**
 * GitHub Copilot Adapter - Adapter for GitHub Copilot Extensions API
 * Copilot supports MCP protocol and has an Extensions SDK
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.CopilotAdapter = void 0;
class CopilotAdapter {
    constructor(config = {}) {
        this.config = {
            spyCodePath: config.spyCodePath || 'spy-code',
            dbPath: config.dbPath || '.spy-code/graph.db',
            enableMCP: config.enableMCP !== false,
            enableHooks: config.enableHooks !== false,
            cacheEnabled: config.cacheEnabled !== false,
            cacheTTL: config.cacheTTL || 300000,
            maxCacheSize: config.maxCacheSize || 1000
        };
        this.copilotExtensionId = 'spy-code-integration';
        this.workspacePath = process.cwd();
    }
    /**
     * Initialize the adapter
     */
    async initialize() {
        console.log('Initializing GitHub Copilot adapter');
        // Setup Copilot extension configuration
        await this.setupCopilotConfig();
        console.log('GitHub Copilot adapter initialized');
    }
    /**
     * Activate the adapter
     */
    async activate() {
        console.log('Activating GitHub Copilot adapter');
        // Register Copilot skill/tool
        await this.registerCopilotSkill();
    }
    /**
     * Deactivate the adapter
     */
    async deactivate() {
        console.log('Deactivating GitHub Copilot adapter');
    }
    /**
     * Show the spy-code panel (not applicable in Copilot)
     */
    showPanel() {
        console.log('Panel not applicable in Copilot');
    }
    /**
     * Hide the spy-code panel (not applicable in Copilot)
     */
    hidePanel() {
        console.log('Panel not applicable in Copilot');
    }
    /**
     * Show a notification
     */
    showNotification(message, type) {
        // Copilot uses its own notification system
        console.log(`[${type}] ${message}`);
    }
    /**
     * Get the current file
     */
    getCurrentFile() {
        // Copilot tracks the current file in its context
        return null;
    }
    /**
     * Get the current selection
     */
    getCurrentSelection() {
        // Copilot tracks selection in its context
        return null;
    }
    /**
     * Go to definition at a specific location
     */
    async goToDefinition(filePath, line) {
        // Copilot can navigate to locations
        console.log(`Navigate to ${filePath}:${line}`);
    }
    /**
     * Show references
     */
    async showReferences(references) {
        console.log(`Found ${references.length} references:`);
        for (const ref of references) {
            console.log(`  - ${ref.node.name} at ${ref.filePath}:${ref.line}`);
        }
    }
    /**
     * Get the current configuration
     */
    getConfig() {
        return { ...this.config };
    }
    /**
     * Update the configuration
     */
    updateConfig(config) {
        this.config = { ...this.config, ...config };
    }
    /**
     * Setup Copilot extension configuration
     */
    async setupCopilotConfig() {
        // Copilot extensions use MCP configuration
        const mcpConfig = {
            mcpServers: {
                'spy-code': {
                    command: this.config.spyCodePath,
                    args: ['serve', '--mcp'],
                    env: {
                        SPY_DB_PATH: this.config.dbPath
                    }
                }
            }
        };
        console.log('Copilot MCP configuration:', JSON.stringify(mcpConfig, null, 2));
    }
    /**
     * Register Copilot skill/tool
     */
    async registerCopilotSkill() {
        // Copilot skills are registered via the Extensions API
        const skill = {
            id: this.copilotExtensionId,
            name: 'Spy-Code Integration',
            description: 'Search and navigate code using spy-code',
            version: '0.1.0',
            capabilities: {
                tools: [
                    {
                        name: 'spy-code-search',
                        description: 'Search the codebase',
                        inputSchema: {
                            type: 'object',
                            properties: {
                                query: { type: 'string' },
                                kind: { type: 'string' }
                            }
                        }
                    },
                    {
                        name: 'spy-code-semantic-search',
                        description: 'Semantic search using embeddings',
                        inputSchema: {
                            type: 'object',
                            properties: {
                                query: { type: 'string' }
                            }
                        }
                    },
                    {
                        name: 'spy-code-get-node',
                        description: 'Get node details',
                        inputSchema: {
                            type: 'object',
                            properties: {
                                nodeId: { type: 'string' }
                            }
                        }
                    },
                    {
                        name: 'spy-code-callers',
                        description: 'Find callers of a function',
                        inputSchema: {
                            type: 'object',
                            properties: {
                                nodeId: { type: 'string' },
                                depth: { type: 'number' }
                            }
                        }
                    },
                    {
                        name: 'spy-code-callees',
                        description: 'Find callees of a function',
                        inputSchema: {
                            type: 'object',
                            properties: {
                                nodeId: { type: 'string' },
                                depth: { type: 'number' }
                            }
                        }
                    }
                ]
            }
        };
        console.log('Registered Copilot skill:', skill.name);
    }
    /**
     * Get workspace path
     */
    getWorkspacePath() {
        return this.workspacePath;
    }
    /**
     * Set workspace path
     */
    setWorkspacePath(path) {
        this.workspacePath = path;
    }
    /**
     * Get Copilot extension ID
     */
    getExtensionId() {
        return this.copilotExtensionId;
    }
}
exports.CopilotAdapter = CopilotAdapter;
