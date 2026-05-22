/**
 * Error Recovery Hook Implementation
 * Provides graceful error handling for spy-code operations
 */
import { HookContext, HookResult } from '../../types';
export declare class ErrorRecoveryHook {
    private maxRetries;
    private retryDelay;
    constructor(maxRetries?: number, retryDelay?: number);
    /**
     * Create a hook handler that recovers from MCP tool errors
     */
    createMcpErrorHandler(): (context: HookContext) => Promise<HookResult>;
}
