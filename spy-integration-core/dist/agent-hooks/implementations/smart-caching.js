"use strict";
/**
 * Smart Caching Hook Implementation
 * Intelligently caches spy-code data based on usage patterns
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.SmartCachingHook = void 0;
const types_1 = require("../../types");
class SmartCachingHook {
    constructor(cacheManager, cacheTTL = 300000) {
        this.cacheManager = cacheManager;
        this.cacheTTL = cacheTTL;
    }
}
exports.SmartCachingHook = SmartCachingHook;
{
    return async (context) => {
        if (!context.node?.id) {
            return { continue: true };
        }
        try {
            const cacheKey = `node:${context.node.id}`;
            // Check if already cached
            const cached = this.cacheManager.get(cacheKey);
            if (cached) {
                context.metadata = {
                    ...context.metadata,
                    cachedNode: cached
                };
                return { continue: true };
            }
            // Cache the node data if available
            if (context.metadata?.nodeInfo) {
                this.cacheManager.set(cacheKey, context.metadata.nodeInfo, this.cacheTTL);
            }
        }
        catch (error) {
            console.error('[SmartCachingHook] Failed to cache node:', error);
        }
        return { continue: true };
    };
}
/**
 * Create a hook handler that caches search results
 */
createSearchCacheHandler();
(context) => Promise;
{
    return async (context) => {
        // Cache search results if available in metadata
        if (context.metadata?.searchResults && context.metadata?.searchQuery) {
            try {
                const cacheKey = `search:${context.metadata.searchQuery}`;
                this.cacheManager.set(cacheKey, context.metadata.searchResults, this.cacheTTL);
            }
            catch (error) {
                console.error('[SmartCachingHook] Failed to cache search:', error);
            }
        }
        return { continue: true };
    };
}
/**
 * Create a hook handler that invalidates cache on code changes
 */
createCacheInvalidationHandler();
(context) => Promise;
{
    return async (context) => {
        try {
            // Clear cache for the modified file
            if (context.filePath) {
                const pattern = new RegExp(`node:${context.filePath.replace(/\//g, ':')}:.*`);
                this.cacheManager.clearPattern(pattern);
            }
            // Alternatively, clear all cache on any code change
            // this.cacheManager.clear();
        }
        catch (error) {
            console.error('[SmartCachingHook] Failed to invalidate cache:', error);
        }
        return { continue: true };
    };
}
/**
 * Register the smart caching hooks
 */
register(hooks, any);
void {
    hooks, : .registerHook(types_1.HookType.POST_READ_CODE, this.createNodeCacheHandler()),
    hooks, : .registerHook(types_1.HookType.POST_MCP_TOOL_USE, this.createSearchCacheHandler()),
    hooks, : .registerHook(types_1.HookType.POST_WRITE_CODE, this.createCacheInvalidationHandler())
};
