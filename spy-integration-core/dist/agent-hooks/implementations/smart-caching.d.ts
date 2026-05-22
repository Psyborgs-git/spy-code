/**
 * Smart Caching Hook Implementation
 * Intelligently caches spy-code data based on usage patterns
 */
import { HookContext, HookResult } from '../../types';
import { CacheManager } from '../../cache-manager';
export declare class SmartCachingHook {
    private cacheManager;
    private cacheTTL;
    constructor(cacheManager: CacheManager, cacheTTL?: number);
    /**
     * Create a hook handler that caches node data
     */
    createNodeCacheHandler(): (context: HookContext) => Promise<HookResult>;
}
