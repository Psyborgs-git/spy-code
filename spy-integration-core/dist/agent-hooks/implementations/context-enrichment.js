"use strict";
/**
 * Context Enrichment Hook Implementation
 * Enriches agent context with graph data from spy-code
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.ContextEnrichmentHook = void 0;
const types_1 = require("../../types");
class ContextEnrichmentHook {
    constructor(mcpClient, maxDepth = 2) {
        this.mcpClient = mcpClient;
        this.maxDepth = maxDepth;
    }
    /**
     * Create a hook handler that enriches context before reading code
     */
    createPreReadHandler() {
        return async (context) => {
            if (!context.node?.id) {
                return { continue: true };
            }
            try {
                // Get node details
                const node = await this.mcpClient.getNode(context.node.id);
                if (!node) {
                    return { continue: true };
                }
                // Get callers and callees
                const [callers, callees] = await Promise.all([
                    this.mcpClient.getCallers(context.node.id, this.maxDepth),
                    this.mcpClient.getCallees(context.node.id, this.maxDepth)
                ]);
                // Enrich context
                context.metadata = {
                    ...context.metadata,
                    nodeInfo: node,
                    callers: callers,
                    callees: callees,
                    callGraphDepth: this.maxDepth
                };
                return { continue: true };
            }
            catch (error) {
                console.error('[ContextEnrichmentHook] Failed to enrich context:', error);
                // Don't block on enrichment errors
                return { continue: true };
            }
        };
    }
    /**
     * Create a hook handler that enriches context after reading code
     */
    createPostReadHandler() {
        return async (context) => {
            // Cache the enriched context for future use
            if (context.metadata?.nodeInfo) {
                // Could integrate with CacheManager here
                console.log('[ContextEnrichmentHook] Context enriched and cached');
            }
            return { continue: true };
        };
    }
    /**
     * Register the context enrichment hooks
     */
    register(hooks) {
        hooks.registerHook(types_1.HookType.PRE_READ_CODE, this.createPreReadHandler());
        hooks.registerHook(types_1.HookType.POST_READ_CODE, this.createPostReadHandler());
    }
}
exports.ContextEnrichmentHook = ContextEnrichmentHook;
