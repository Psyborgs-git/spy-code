"use strict";
/**
 * CLI Bridge - Wrapper around spy-code CLI commands
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.CLIBridge = void 0;
const child_process_1 = require("child_process");
const types_1 = require("../types");
class CLIBridge {
    constructor(config = {}) {
        this.config = {
            command: config.command || 'spy-code',
            args: config.args || [],
            timeout: config.timeout || 30000,
            env: config.env || process.env
        };
    }
    /**
     * Execute a spy-code CLI command
     */
    async executeCommand(args, options = {}) {
        const { timeout = this.config.timeout, cwd } = options;
        const fullArgs = [...this.config.args, ...args];
        return new Promise((resolve, reject) => {
            const child = (0, child_process_1.spawn)(this.config.command, fullArgs, {
                env: this.config.env,
                cwd: cwd || process.cwd()
            });
            let stdout = '';
            child.stdout?.on('data', (data) => {
                stdout += data.toString();
            });
            child.stderr?.on('data', () => {
                // Ignore stderr for now
            });
            const timer = setTimeout(() => {
                child.kill();
                reject(new types_1.CLIError(`Command timed out after ${timeout}ms`));
            }, timeout);
            child.on('close', (code) => {
                clearTimeout(timer);
                if (code === 0) {
                    resolve(stdout);
                }
                else {
                    reject(new types_1.CLIError(`Command failed with exit code ${code}`));
                }
            });
            child.on('error', (error) => {
                clearTimeout(timer);
                reject(new types_1.CLIError(`Failed to spawn command: ${error.message}`, { error }));
            });
        });
    }
    /**
     * Search for nodes by name/description
     */
    async search(query, options = {}) {
        const args = ['search', query];
        if (options.kind) {
            args.push('--kind', options.kind);
        }
        args.push('--json');
        try {
            const output = await this.executeCommand(args);
            const data = JSON.parse(output);
            return data.results || [];
        }
        catch (error) {
            throw new types_1.CLIError(`Search failed: ${error}`, { error });
        }
    }
    /**
     * Semantic search using embeddings
     */
    async semanticSearch(query, options = {}) {
        const args = ['search', query, '--semantic', '--json'];
        if (options.limit) {
            args.push('--limit', options.limit.toString());
        }
        try {
            const output = await this.executeCommand(args);
            const data = JSON.parse(output);
            return data.results || [];
        }
        catch (error) {
            throw new types_1.CLIError(`Semantic search failed: ${error}`, { error });
        }
    }
    /**
     * Get a specific node by ID
     */
    async getNode(nodeId) {
        try {
            const output = await this.executeCommand(['get', nodeId, '--json']);
            const data = JSON.parse(output);
            return data.node || null;
        }
        catch (error) {
            if (error instanceof types_1.CLIError && error.message.includes('not found')) {
                return null;
            }
            throw new types_1.CLIError(`Get node failed: ${error}`, { error });
        }
    }
    /**
     * Get callers of a node
     */
    async getCallers(nodeId, depth = 1) {
        try {
            const output = await this.executeCommand(['callers', nodeId, '--depth', depth.toString(), '--json']);
            const data = JSON.parse(output);
            return data.edges || [];
        }
        catch (error) {
            throw new types_1.CLIError(`Get callers failed: ${error}`, { error });
        }
    }
    /**
     * Get callees of a node
     */
    async getCallees(nodeId, depth = 1) {
        try {
            const output = await this.executeCommand(['callees', nodeId, '--depth', depth.toString(), '--json']);
            const data = JSON.parse(output);
            return data.edges || [];
        }
        catch (error) {
            throw new types_1.CLIError(`Get callees failed: ${error}`, { error });
        }
    }
    /**
     * Get graph data with optional filters
     */
    async getGraphData(filter) {
        const args = ['query', '--json'];
        let query = '{ graphData { nodes { id kind name language filePath startLine endLine } edges { fromId toId kind confidence } } }';
        if (filter) {
            // Build filter query
            const filters = [];
            if (filter.filePath)
                filters.push(`filePath: "${filter.filePath}"`);
            if (filter.nodeKinds?.length)
                filters.push(`nodeKinds: [${filter.nodeKinds.map(k => `"${k}"`).join(', ')}]`);
            if (filter.languages?.length)
                filters.push(`languages: [${filter.languages.map(l => `"${l}"`).join(', ')}]`);
            if (filter.edgeKinds?.length)
                filters.push(`edgeKinds: [${filter.edgeKinds.map(k => `"${k}"`).join(', ')}]`);
            if (filters.length) {
                query = `{ graphData(filter: { ${filters.join(', ')} }) { nodes { id kind name language filePath startLine endLine } edges { fromId toId kind confidence } } }`;
            }
        }
        args.push(query);
        try {
            const output = await this.executeCommand(args);
            const data = JSON.parse(output);
            return data.data?.graphData || { nodes: [], edges: [] };
        }
        catch (error) {
            throw new types_1.CLIError(`Get graph data failed: ${error}`, { error });
        }
    }
    /**
     * Get index statistics
     */
    async getStats() {
        try {
            const output = await this.executeCommand(['stats', '--json']);
            const data = JSON.parse(output);
            return data.stats || { nodeCount: 0, edgeCount: 0, fileCount: 0 };
        }
        catch (error) {
            throw new types_1.CLIError(`Get stats failed: ${error}`, { error });
        }
    }
    /**
     * Reindex the codebase
     */
    async reindex(full = false) {
        const args = ['index'];
        if (full) {
            args.push('--full');
        }
        args.push('--json');
        try {
            const output = await this.executeCommand(args);
            const data = JSON.parse(output);
            return data.stats || { nodeCount: 0, edgeCount: 0, fileCount: 0 };
        }
        catch (error) {
            throw new types_1.CLIError(`Reindex failed: ${error}`, { error });
        }
    }
    /**
     * Get nodes changed since a git ref
     */
    async changedSince(ref) {
        try {
            const output = await this.executeCommand(['changed', ref, '--json']);
            const data = JSON.parse(output);
            return data.nodes || [];
        }
        catch (error) {
            throw new types_1.CLIError(`Changed since failed: ${error}`, { error });
        }
    }
    /**
     * Generate embeddings for semantic search
     */
    async generateEmbeddings(full = false) {
        const args = ['embed'];
        if (full) {
            args.push('--full');
        }
        try {
            await this.executeCommand(args);
        }
        catch (error) {
            throw new types_1.CLIError(`Generate embeddings failed: ${error}`, { error });
        }
    }
    /**
     * Ask a natural language question
     */
    async ask(query, limit = 20) {
        const args = ['ask', query, '--limit', limit.toString(), '--json'];
        try {
            const output = await this.executeCommand(args);
            const data = JSON.parse(output);
            return data.results || [];
        }
        catch (error) {
            throw new types_1.CLIError(`Ask failed: ${error}`, { error });
        }
    }
    /**
     * Initialize spy-code in a directory
     */
    async init(cwd) {
        try {
            await this.executeCommand(['init'], { cwd });
        }
        catch (error) {
            throw new types_1.CLIError(`Init failed: ${error}`, { error });
        }
    }
    /**
     * Check if spy-code is available
     */
    async isAvailable() {
        try {
            await this.executeCommand(['--version']);
            return true;
        }
        catch {
            return false;
        }
    }
    /**
     * Update CLI configuration
     */
    updateConfig(config) {
        this.config = { ...this.config, ...config };
    }
}
exports.CLIBridge = CLIBridge;
