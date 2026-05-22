/**
 * MCP Adapter - Model Context Protocol client implementation
 */
import { EventEmitter } from 'events';
import { Node, SearchResult, Edge, IndexStats, SearchOptions } from '../types';
interface MCPTool {
    name: string;
    description: string;
    inputSchema: any;
}
interface MCPResource {
    uri: string;
    name: string;
    description: string;
    mimeType: string;
}
export declare class MCPClient extends EventEmitter {
    private process;
    private requestId;
    private pendingRequests;
    private isConnected;
    private configPath;
    constructor(configPath?: string);
    /**
     * Connect to MCP server
     */
    connect(): Promise<void>;
    /**
     * Process incoming messages from MCP server
     */
    private processMessages;
    /**
     * Handle MCP response
     */
    private handleResponse;
    /**
     * Send request to MCP server
     */
    private sendRequest;
    /**
     * Initialize MCP session
     */
    initialize(): Promise<void>;
    /**
     * List available tools
     */
    listTools(): Promise<MCPTool[]>;
    /**
     * Call a tool
     */
    callTool(name: string, args: any): Promise<any>;
    /**
     * List available resources
     */
    listResources(): Promise<MCPResource[]>;
    /**
     * Read a resource
     */
    readResource(uri: string): Promise<any>;
    /**
     * Search nodes using MCP
     */
    search(query: string, options?: SearchOptions): Promise<SearchResult[]>;
    /**
     * Get node using MCP
     */
    getNode(nodeId: string): Promise<Node | null>;
    /**
     * Get callers using MCP
     */
    getCallers(nodeId: string, depth?: number): Promise<Edge[]>;
    /**
     * Get callees using MCP
     */
    getCallees(nodeId: string, depth?: number): Promise<Edge[]>;
    /**
     * Get changed nodes using MCP
     */
    changedSince(ref: string): Promise<Node[]>;
    /**
     * Get stats using MCP
     */
    getStats(): Promise<IndexStats>;
    /**
     * Run GraphQL query using MCP
     */
    queryGraph(query: string, variables?: any): Promise<any>;
    /**
     * Generate embeddings using MCP
     */
    generateEmbeddings(full?: boolean): Promise<void>;
    /**
     * Ask natural language question using MCP
     */
    ask(query: string, limit?: number): Promise<SearchResult[]>;
    /**
     * Disconnect from MCP server
     */
    disconnect(): Promise<void>;
    /**
     * Check if connected
     */
    connected(): boolean;
}
export {};
