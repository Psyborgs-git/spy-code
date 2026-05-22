"use strict";
/**
 * MCP Adapter - Model Context Protocol client implementation
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.MCPClient = void 0;
const child_process_1 = require("child_process");
const events_1 = require("events");
const types_1 = require("../types");
class MCPClient extends events_1.EventEmitter {
    constructor(configPath = 'spy.config.json') {
        super();
        this.process = null;
        this.requestId = 0;
        this.pendingRequests = new Map();
        this.isConnected = false;
        this.configPath = configPath;
    }
    /**
     * Connect to MCP server
     */
    async connect() {
        if (this.isConnected) {
            return;
        }
        return new Promise((resolve, reject) => {
            this.process = (0, child_process_1.spawn)('spy-code', ['serve', '--mcp'], {
                env: process.env
            });
            let stdout = '';
            this.process.stdout?.on('data', (data) => {
                stdout += data.toString();
                this.processMessages(stdout);
                stdout = '';
            });
            this.process.stderr?.on('data', (data) => {
                // Log stderr for debugging
                console.error('MCP stderr:', data.toString());
            });
            this.process.on('error', (error) => {
                reject(new types_1.MCPError(`Failed to start MCP server: ${error.message}`, { error }));
            });
            this.process.on('close', (code) => {
                this.isConnected = false;
                this.emit('close', code);
                if (code !== 0 && this.pendingRequests.size > 0) {
                    const error = new types_1.MCPError(`MCP server closed with code ${code}`);
                    this.pendingRequests.forEach(({ reject }) => reject(error));
                    this.pendingRequests.clear();
                }
            });
            // Wait for initialization
            setTimeout(() => {
                this.isConnected = true;
                this.emit('connected');
                resolve();
            }, 500);
        });
    }
    /**
     * Process incoming messages from MCP server
     */
    processMessages(data) {
        const lines = data.trim().split('\n');
        for (const line of lines) {
            if (!line.trim())
                continue;
            try {
                const message = JSON.parse(line);
                this.handleResponse(message);
            }
            catch (error) {
                this.emit('error', new types_1.MCPError(`Failed to parse MCP message: ${error}`, { error, line }));
            }
        }
    }
    /**
     * Handle MCP response
     */
    handleResponse(response) {
        const pending = this.pendingRequests.get(response.id);
        if (!pending) {
            return;
        }
        this.pendingRequests.delete(response.id);
        if (response.error) {
            pending.reject(new types_1.MCPError(response.error.message, response.error));
        }
        else {
            pending.resolve(response.result);
        }
    }
    /**
     * Send request to MCP server
     */
    async sendRequest(method, params) {
        if (!this.isConnected) {
            throw new types_1.MCPError('MCP client is not connected');
        }
        const id = ++this.requestId;
        const request = {
            jsonrpc: '2.0',
            id,
            method,
            params
        };
        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject });
            const message = JSON.stringify(request) + '\n';
            this.process?.stdin?.write(message);
            // Timeout after 30 seconds
            setTimeout(() => {
                if (this.pendingRequests.has(id)) {
                    this.pendingRequests.delete(id);
                    reject(new types_1.MCPError(`Request timeout: ${method}`));
                }
            }, 30000);
        });
    }
    /**
     * Initialize MCP session
     */
    async initialize() {
        await this.sendRequest('initialize', {
            protocolVersion: '2024-11-05',
            capabilities: {
                tools: {},
                resources: {}
            },
            clientInfo: {
                name: 'spy-code-integration',
                version: '0.1.0'
            }
        });
        await this.sendRequest('initialized', {});
    }
    /**
     * List available tools
     */
    async listTools() {
        const response = await this.sendRequest('tools/list');
        return response.tools || [];
    }
    /**
     * Call a tool
     */
    async callTool(name, args) {
        const response = await this.sendRequest('tools/call', {
            name,
            arguments: args
        });
        // Parse content from response
        if (response.content && Array.isArray(response.content)) {
            const textContent = response.content.find((c) => c.type === 'text');
            if (textContent) {
                try {
                    return JSON.parse(textContent.text);
                }
                catch {
                    return textContent.text;
                }
            }
        }
        return response;
    }
    /**
     * List available resources
     */
    async listResources() {
        const response = await this.sendRequest('resources/list');
        return response.resources || [];
    }
    /**
     * Read a resource
     */
    async readResource(uri) {
        const response = await this.sendRequest('resources/read', {
            uri
        });
        if (response.contents && Array.isArray(response.contents)) {
            const content = response.contents[0];
            if (content.mimeType === 'application/json') {
                try {
                    return JSON.parse(content.text);
                }
                catch {
                    return content.text;
                }
            }
            return content.text;
        }
        return response;
    }
    /**
     * Search nodes using MCP
     */
    async search(query, options = {}) {
        const result = await this.callTool('search', {
            query,
            kind: options.kind,
            limit: options.limit
        });
        return result || [];
    }
    /**
     * Get node using MCP
     */
    async getNode(nodeId) {
        const result = await this.callTool('get_node', {
            node_id: nodeId
        });
        try {
            return JSON.parse(result);
        }
        catch {
            return null;
        }
    }
    /**
     * Get callers using MCP
     */
    async getCallers(nodeId, depth = 1) {
        const result = await this.callTool('find_callers', {
            node_id: nodeId,
            depth
        });
        try {
            return JSON.parse(result);
        }
        catch {
            return [];
        }
    }
    /**
     * Get callees using MCP
     */
    async getCallees(nodeId, depth = 1) {
        const result = await this.callTool('find_callees', {
            node_id: nodeId,
            depth
        });
        try {
            return JSON.parse(result);
        }
        catch {
            return [];
        }
    }
    /**
     * Get changed nodes using MCP
     */
    async changedSince(ref) {
        const result = await this.callTool('changed_since', {
            git_ref: ref
        });
        try {
            return JSON.parse(result);
        }
        catch {
            return [];
        }
    }
    /**
     * Get stats using MCP
     */
    async getStats() {
        const result = await this.callTool('stats', {});
        try {
            return JSON.parse(result);
        }
        catch {
            return { nodeCount: 0, edgeCount: 0, fileCount: 0 };
        }
    }
    /**
     * Run GraphQL query using MCP
     */
    async queryGraph(query, variables) {
        const result = await this.callTool('query_graph', {
            query,
            variables
        });
        try {
            return JSON.parse(result);
        }
        catch {
            return result;
        }
    }
    /**
     * Generate embeddings using MCP
     */
    async generateEmbeddings(full = false) {
        await this.callTool('embed', { full });
    }
    /**
     * Ask natural language question using MCP
     */
    async ask(query, limit = 20) {
        const result = await this.callTool('ask', {
            query,
            limit
        });
        try {
            return JSON.parse(result);
        }
        catch {
            return [];
        }
    }
    /**
     * Disconnect from MCP server
     */
    async disconnect() {
        if (this.process) {
            this.process.kill();
            this.process = null;
        }
        this.isConnected = false;
        this.pendingRequests.clear();
        this.emit('disconnected');
    }
    /**
     * Check if connected
     */
    connected() {
        return this.isConnected;
    }
}
exports.MCPClient = MCPClient;
