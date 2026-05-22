/**
 * Agent Hooks Interface - Coding agent lifecycle hooks
 */
import { EventEmitter } from 'eventemitter3';
import { HookType, HookContext, HookHandler, HookResult } from '../types';
export declare class AgentHooks extends EventEmitter {
    private hooks;
    private enabled;
    /**
     * Register a hook handler
     */
    registerHook(hookType: HookType, handler: HookHandler): void;
    /**
     * Unregister a hook handler
     */
    unregisterHook(hookType: HookType, handler: HookHandler): void;
    /**
     * Execute all handlers for a hook type
     */
    executeHook(hookType: HookType, context: HookContext): Promise<HookResult>;
    /**
     * Enable hooks
     */
    enable(): void;
    /**
     * Disable hooks
     */
    disable(): void;
    /**
     * Check if hooks are enabled
     */
    isEnabled(): boolean;
    /**
     * Get all registered hooks
     */
    getRegisteredHooks(): Map<HookType, number>;
    /**
     * Clear all hooks
     */
    clearAll(): void;
    /**
     * Clear hooks for a specific type
     */
    clearType(hookType: HookType): void;
    /**
     * Pre-read code hook
     */
    preReadCode(context: HookContext): Promise<HookResult>;
    /**
     * Post-read code hook
     */
    postReadCode(context: HookContext): Promise<HookResult>;
    /**
     * Pre-write code hook
     */
    preWriteCode(context: HookContext): Promise<HookResult>;
    /**
     * Post-write code hook
     */
    postWriteCode(context: HookContext): Promise<HookResult>;
    /**
     * Pre-run command hook
     */
    preRunCommand(context: HookContext): Promise<HookResult>;
    /**
     * Post-run command hook
     */
    postRunCommand(context: HookContext): Promise<HookResult>;
    /**
     * Pre-MCP tool use hook
     */
    preMcpToolUse(context: HookContext): Promise<HookResult>;
    /**
     * Post-MCP tool use hook
     */
    postMcpToolUse(context: HookContext): Promise<HookResult>;
    /**
     * Pre-user prompt hook
     */
    preUserPrompt(context: HookContext): Promise<HookResult>;
    /**
     * Post-cascade response hook
     */
    postCascadeResponse(context: HookContext): Promise<HookResult>;
    /**
     * Post-setup worktree hook
     */
    postSetupWorktree(context: HookContext): Promise<HookResult>;
}
export declare function getAgentHooks(): AgentHooks;
export declare function resetAgentHooks(): void;
