/**
 * Context Enrichment Hook Implementation
 * Enriches agent context with graph data from spy-code
 */
import { HookContext, HookResult } from '../../types';
import { MCPClient } from '../../mcp-adapter/client';
export declare class ContextEnrichmentHook {
    private mcpClient;
    private maxDepth;
    constructor(mcpClient: MCPClient, maxDepth?: number);
    /**
     * Create a hook handler that enriches context before reading code
     */
    createPreReadHandler(): (context: HookContext) => Promise<HookResult>;
    /**
     * Create a hook handler that enriches context after reading code
     */
    createPostReadHandler(): (context: HookContext) => Promise<HookResult>;
    /**
     * Register the context enrichment hooks
     */
    register(hooks: any): void;
}
