/**
 * CLI Bridge - Wrapper around spy-code CLI commands
 */
import { Node, SearchResult, Edge, IndexStats, GraphData, GraphFilter, SearchOptions, SemanticSearchOptions, CLIConfig } from '../types';
export declare class CLIBridge {
    private config;
    constructor(config?: Partial<CLIConfig>);
    /**
     * Execute a spy-code CLI command
     */
    private executeCommand;
    /**
     * Search for nodes by name/description
     */
    search(query: string, options?: SearchOptions): Promise<SearchResult[]>;
    /**
     * Semantic search using embeddings
     */
    semanticSearch(query: string, options?: SemanticSearchOptions): Promise<SearchResult[]>;
    /**
     * Get a specific node by ID
     */
    getNode(nodeId: string): Promise<Node | null>;
    /**
     * Get callers of a node
     */
    getCallers(nodeId: string, depth?: number): Promise<Edge[]>;
    /**
     * Get callees of a node
     */
    getCallees(nodeId: string, depth?: number): Promise<Edge[]>;
    /**
     * Get graph data with optional filters
     */
    getGraphData(filter?: GraphFilter): Promise<GraphData>;
    /**
     * Get index statistics
     */
    getStats(): Promise<IndexStats>;
    /**
     * Reindex the codebase
     */
    reindex(full?: boolean): Promise<IndexStats>;
    /**
     * Get nodes changed since a git ref
     */
    changedSince(ref: string): Promise<Node[]>;
    /**
     * Generate embeddings for semantic search
     */
    generateEmbeddings(full?: boolean): Promise<void>;
    /**
     * Ask a natural language question
     */
    ask(query: string, limit?: number): Promise<SearchResult[]>;
    /**
     * Initialize spy-code in a directory
     */
    init(cwd?: string): Promise<void>;
    /**
     * Check if spy-code is available
     */
    isAvailable(): Promise<boolean>;
    /**
     * Update CLI configuration
     */
    updateConfig(config: Partial<CLIConfig>): void;
}
