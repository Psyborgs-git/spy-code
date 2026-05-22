"use strict";
/**
 * Auto-Index Hook Implementation
 * Automatically re-indexes the codebase after code changes
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.AutoIndexHook = void 0;
const types_1 = require("../../types");
class AutoIndexHook {
    constructor(mcpClient, debounceMs = 1000) {
        this.timeoutId = null;
        this.mcpClient = mcpClient;
        this.debounceMs = debounceMs;
    }
    /**
     * Create a hook handler that auto-indexes after code changes
     */
    createHandler() {
        return async (context) => {
            // Clear previous timeout
            if (this.timeoutId) {
                clearTimeout(this.timeoutId);
            }
            // Debounce the indexing
            this.timeoutId = setTimeout(async () => {
                try {
                    await this.mcpClient.callTool('index', {});
                    console.log('[AutoIndexHook] Codebase re-indexed after code change');
                }
                catch (error) {
                    console.error('[AutoIndexHook] Failed to re-index:', error);
                    // Don't block on indexing errors
                }
            }, this.debounceMs);
            return { continue: true };
        };
    }
    /**
     * Register the auto-index hook for post-write code events
     */
    register(hooks) {
        hooks.registerHook(types_1.HookType.POST_WRITE_CODE, this.createHandler());
    }
}
exports.AutoIndexHook = AutoIndexHook;
