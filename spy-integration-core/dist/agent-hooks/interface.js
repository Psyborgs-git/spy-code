"use strict";
/**
 * Agent Hooks Interface - Coding agent lifecycle hooks
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.AgentHooks = void 0;
exports.getAgentHooks = getAgentHooks;
exports.resetAgentHooks = resetAgentHooks;
const eventemitter3_1 = require("eventemitter3");
const types_1 = require("../types");
class AgentHooks extends eventemitter3_1.EventEmitter {
    constructor() {
        super(...arguments);
        this.hooks = new Map();
        this.enabled = true;
    }
    /**
     * Register a hook handler
     */
    registerHook(hookType, handler) {
        if (!this.hooks.has(hookType)) {
            this.hooks.set(hookType, new Set());
        }
        this.hooks.get(hookType).add(handler);
        this.emit('hook_registered', { hookType, handler });
    }
    /**
     * Unregister a hook handler
     */
    unregisterHook(hookType, handler) {
        const handlers = this.hooks.get(hookType);
        if (handlers) {
            handlers.delete(handler);
            if (handlers.size === 0) {
                this.hooks.delete(hookType);
            }
            this.emit('hook_unregistered', { hookType, handler });
        }
    }
    /**
     * Execute all handlers for a hook type
     */
    async executeHook(hookType, context) {
        if (!this.enabled) {
            return { continue: true };
        }
        const handlers = this.hooks.get(hookType);
        if (!handlers || handlers.size === 0) {
            return { continue: true };
        }
        const result = { continue: true };
        const modifications = [];
        const metadata = {};
        for (const handler of handlers) {
            try {
                const handlerResult = await handler(context);
                // If any handler says to stop, stop execution
                if (!handlerResult.continue) {
                    result.continue = false;
                }
                // Collect modifications
                if (handlerResult.modifications) {
                    modifications.push(...handlerResult.modifications);
                }
                // Collect metadata
                if (handlerResult.metadata) {
                    Object.assign(metadata, handlerResult.metadata);
                }
                // If handler returned an error, propagate it
                if (handlerResult.error) {
                    result.error = handlerResult.error;
                    break;
                }
                this.emit('hook_executed', { hookType, handler, result: handlerResult });
            }
            catch (error) {
                result.error = error;
                result.continue = false;
                this.emit('hook_error', { hookType, handler, error });
                break;
            }
        }
        if (modifications.length > 0) {
            result.modifications = modifications;
        }
        if (Object.keys(metadata).length > 0) {
            result.metadata = metadata;
        }
        return result;
    }
    /**
     * Enable hooks
     */
    enable() {
        this.enabled = true;
        this.emit('hooks_enabled');
    }
    /**
     * Disable hooks
     */
    disable() {
        this.enabled = false;
        this.emit('hooks_disabled');
    }
    /**
     * Check if hooks are enabled
     */
    isEnabled() {
        return this.enabled;
    }
    /**
     * Get all registered hooks
     */
    getRegisteredHooks() {
        const result = new Map();
        for (const [type, handlers] of this.hooks.entries()) {
            result.set(type, handlers.size);
        }
        return result;
    }
    /**
     * Clear all hooks
     */
    clearAll() {
        this.hooks.clear();
        this.emit('hooks_cleared');
    }
    /**
     * Clear hooks for a specific type
     */
    clearType(hookType) {
        this.hooks.delete(hookType);
        this.emit('hooks_cleared_type', { hookType });
    }
    /**
     * Pre-read code hook
     */
    async preReadCode(context) {
        return this.executeHook(types_1.HookType.PRE_READ_CODE, context);
    }
    /**
     * Post-read code hook
     */
    async postReadCode(context) {
        return this.executeHook(types_1.HookType.POST_READ_CODE, context);
    }
    /**
     * Pre-write code hook
     */
    async preWriteCode(context) {
        return this.executeHook(types_1.HookType.PRE_WRITE_CODE, context);
    }
    /**
     * Post-write code hook
     */
    async postWriteCode(context) {
        return this.executeHook(types_1.HookType.POST_WRITE_CODE, context);
    }
    /**
     * Pre-run command hook
     */
    async preRunCommand(context) {
        return this.executeHook(types_1.HookType.PRE_RUN_COMMAND, context);
    }
    /**
     * Post-run command hook
     */
    async postRunCommand(context) {
        return this.executeHook(types_1.HookType.POST_RUN_COMMAND, context);
    }
    /**
     * Pre-MCP tool use hook
     */
    async preMcpToolUse(context) {
        return this.executeHook(types_1.HookType.PRE_MCP_TOOL_USE, context);
    }
    /**
     * Post-MCP tool use hook
     */
    async postMcpToolUse(context) {
        return this.executeHook(types_1.HookType.POST_MCP_TOOL_USE, context);
    }
    /**
     * Pre-user prompt hook
     */
    async preUserPrompt(context) {
        return this.executeHook(types_1.HookType.PRE_USER_PROMPT, context);
    }
    /**
     * Post-cascade response hook
     */
    async postCascadeResponse(context) {
        return this.executeHook(types_1.HookType.POST_CASCADE_RESPONSE, context);
    }
    /**
     * Post-setup worktree hook
     */
    async postSetupWorktree(context) {
        return this.executeHook(types_1.HookType.POST_SETUP_WORKTREE, context);
    }
}
exports.AgentHooks = AgentHooks;
// Create singleton instance
let agentHooksInstance = null;
function getAgentHooks() {
    if (!agentHooksInstance) {
        agentHooksInstance = new AgentHooks();
    }
    return agentHooksInstance;
}
function resetAgentHooks() {
    if (agentHooksInstance) {
        agentHooksInstance.clearAll();
        agentHooksInstance = null;
    }
}
