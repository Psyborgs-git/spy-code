/**
 * Cache Manager - In-memory and disk-based caching for query results
 */
export declare class CacheManager {
    private memoryCache;
    private diskCacheEnabled;
    private diskCacheDir;
    private defaultTTL;
    private maxCacheSize;
    constructor(options?: {
        maxCacheSize?: number;
        defaultTTL?: number;
        diskCacheEnabled?: boolean;
        diskCacheDir?: string;
    });
    /**
     * Initialize disk cache directory
     */
    private initializeDiskCache;
    /**
     * Generate cache key
     */
    private generateKey;
    /**
     * Get from cache
     */
    get<T>(prefix: string, params: any): Promise<T | null>;
    /**
     * Set in cache
     */
    set<T>(prefix: string, params: any, data: T, ttl?: number): Promise<void>;
    /**
     * Delete from cache
     */
    delete(prefix: string, params: any): Promise<void>;
    /**
     * Clear all cache
     */
    clear(): Promise<void>;
    /**
     * Clear cache by prefix
     */
    clearPrefix(prefix: string): Promise<void>;
    /**
     * Get cache statistics
     */
    getStats(): {
        memorySize: number;
        memoryUsage: number;
        diskEnabled: boolean;
        diskCacheDir: string;
    };
    /**
     * Clean expired entries from disk cache
     */
    cleanExpired(): Promise<void>;
    /**
     * Update configuration
     */
    updateConfig(options: {
        maxCacheSize?: number;
        defaultTTL?: number;
        diskCacheEnabled?: boolean;
    }): void;
}
