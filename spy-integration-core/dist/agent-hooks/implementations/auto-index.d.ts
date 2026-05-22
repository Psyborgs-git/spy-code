/**
 * Auto-Index Hook Implementation
 * Automatically re-indexes the codebase after code changes
 */
import { HookContext, HookResult } from '../../types';
import { MCPClient } from '../../mcp-adapter/client';
export declare class AutoIndexHook {
    private mcpClient;
    private debounceMs;
    private timeoutId;
    constructor(mcpClient: MCPClient, debounceMs?: number);
    /**
     * Create a hook handler that auto-indexes after code changes
     */
    createHandler(): (context: HookContext) => Promise<HookResult>;
    /**
     * Register the auto-index hook for post-write code events
     */
    register(hooks: any): void;
}
