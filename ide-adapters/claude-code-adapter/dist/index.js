"use strict";
/**
 * Claude Code Adapter - Adapter for Claude Code extension API
 * Claude Code has built-in MCP support and workflow system
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.ClaudeCodeAdapter = void 0;
const child_process_1 = require("child_process");
const util_1 = require("util");
const integration_core_1 = require("@spy-code/integration-core");
const execAsync = (0, util_1.promisify)(child_process_1.exec);
class ClaudeCodeAdapter {
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
        this.claudeCodePath = 'claude-code';
        this.workspacePath = process.cwd();
    }
    /**
     * Initialize the adapter
     */
    async initialize() {
        console.log('Initializing Claude Code adapter');
        // Check if Claude Code is available
        try {
            await execAsync(`${this.claudeCodePath} --version`);
        }
        catch (error) {
            console.warn('Claude Code not found in PATH');
        }
        // Setup MCP configuration for Claude Code
        await this.setupMCPConfig();
        console.log('Claude Code adapter initialized');
    }
    /**
     * Activate the adapter
     */
    async activate() {
        console.log('Activating Claude Code adapter');
        // Register skills and workflows
        await this.registerSkills();
        await this.registerWorkflows();
    }
    /**
     * Deactivate the adapter
     */
    async deactivate() {
        console.log('Deactivating Claude Code adapter');
    }
    /**
     * Show the spy-code panel (not applicable in Claude Code)
     */
    showPanel() {
        console.log('Panel not applicable in Claude Code');
    }
    /**
     * Hide the spy-code panel (not applicable in Claude Code)
     */
    hidePanel() {
        console.log('Panel not applicable in Claude Code');
    }
    /**
     * Show a notification
     */
    showNotification(message, type) {
        // Claude Code uses stdout/stderr for notifications
        const prefix = type === integration_core_1.NotificationType.ERROR ? '[ERROR]' :
            type === integration_core_1.NotificationType.WARNING ? '[WARNING]' : '[INFO]';
        console.log(`${prefix} ${message}`);
    }
    /**
     * Get the current file
     */
    getCurrentFile() {
        // Claude Code tracks the current file in its context
        // This would need to be retrieved from Claude Code's state
        return null;
    }
    /**
     * Get the current selection
     */
    getCurrentSelection() {
        // Claude Code tracks selection in its context
        return null;
    }
    /**
     * Go to definition at a specific location
     */
    async goToDefinition(filePath, line) {
        // Claude Code can open files at specific lines
        try {
            await execAsync(`${this.claudeCodePath} open ${filePath}:${line}`);
        }
        catch (error) {
            console.error(`Failed to open file: ${error}`);
        }
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
     * Setup MCP configuration for Claude Code
     */
    async setupMCPConfig() {
        // Claude Code uses MCP configuration in .claude/mcp.json
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
        // Write MCP configuration
        // This would typically be done by the user or during extension installation
        console.log('MCP configuration for spy-code:', JSON.stringify(mcpConfig, null, 2));
    }
    /**
     * Register Claude Code skills
     */
    async registerSkills() {
        // Claude Code skills are defined in .claude/skills/
        // This would create skill files for spy-code operations
        const skills = [
            {
                name: 'spy-code-search',
                description: 'Search the codebase using spy-code',
                parameters: {
                    type: 'object',
                    properties: {
                        query: { type: 'string', description: 'Search query' },
                        kind: { type: 'string', description: 'Node kind filter (function, class, constant)' }
                    }
                }
            },
            {
                name: 'spy-code-semantic-search',
                description: 'Semantic search using embeddings',
                parameters: {
                    type: 'object',
                    properties: {
                        query: { type: 'string', description: 'Natural language query' }
                    }
                }
            },
            {
                name: 'spy-code-get-node',
                description: 'Get detailed information about a node',
                parameters: {
                    type: 'object',
                    properties: {
                        nodeId: { type: 'string', description: 'Node ID' }
                    }
                }
            },
            {
                name: 'spy-code-callers',
                description: 'Find callers of a function',
                parameters: {
                    type: 'object',
                    properties: {
                        nodeId: { type: 'string', description: 'Node ID' },
                        depth: { type: 'number', description: 'Call depth' }
                    }
                }
            },
            {
                name: 'spy-code-callees',
                description: 'Find callees of a function',
                parameters: {
                    type: 'object',
                    properties: {
                        nodeId: { type: 'string', description: 'Node ID' },
                        depth: { type: 'number', description: 'Call depth' }
                    }
                }
            }
        ];
        console.log('Registered Claude Code skills:', skills.map(s => s.name));
    }
    /**
     * Register Claude Code workflows
     */
    async registerWorkflows() {
        // Claude Code workflows are defined in .claude/workflows/
        // This would create workflow templates for common tasks
        const workflows = [
            {
                name: 'analyze-function',
                description: 'Analyze a function and its relationships',
                steps: [
                    'Get function details',
                    'Find callers',
                    'Find callees',
                    'Generate summary'
                ]
            },
            {
                name: 'impact-analysis',
                description: 'Analyze the impact of changing a function',
                steps: [
                    'Get function details',
                    'Find all callers recursively',
                    'Identify affected files',
                    'Generate impact report'
                ]
            },
            {
                name: 'code-review',
                description: 'Review code using spy-code context',
                steps: [
                    'Get relevant nodes',
                    'Check for patterns',
                    'Generate review comments'
                ]
            }
        ];
        console.log('Registered Claude Code workflows:', workflows.map(w => w.name));
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
     * Execute a Claude Code command
     */
    async executeCommand(command, args) {
        try {
            const { stdout } = await execAsync(`${this.claudeCodePath} ${command} ${args.join(' ')}`);
            return stdout;
        }
        catch (error) {
            throw new Error(`Claude Code command failed: ${error}`);
        }
    }
    /**
     * Get Claude Code version
     */
    async getVersion() {
        try {
            const { stdout } = await execAsync(`${this.claudeCodePath} --version`);
            return stdout.trim();
        }
        catch {
            return 'unknown';
        }
    }
}
exports.ClaudeCodeAdapter = ClaudeCodeAdapter;
