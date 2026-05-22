"use strict";
/**
 * Cache Manager - In-memory and disk-based caching for query results
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.CacheManager = void 0;
const crypto_1 = require("crypto");
const fs_1 = require("fs");
const path_1 = require("path");
// Simple LRU cache implementation
class SimpleLRUCache {
    constructor(maxSize) {
        this.cache = new Map();
        this.maxSize = maxSize;
    }
    get(key) {
        const value = this.cache.get(key);
        if (value !== undefined) {
            // Move to end (most recently used)
            this.cache.delete(key);
            this.cache.set(key, value);
        }
        return value;
    }
    set(key, value) {
        if (this.cache.has(key)) {
            this.cache.delete(key);
        }
        else if (this.cache.size >= this.maxSize) {
            // Remove first item (least recently used)
            const firstKey = this.cache.keys().next().value;
            if (firstKey !== undefined) {
                this.cache.delete(firstKey);
            }
        }
        this.cache.set(key, value);
        return true;
    }
    delete(key) {
        return this.cache.delete(key);
    }
    clear() {
        this.cache.clear();
    }
    has(key) {
        return this.cache.has(key);
    }
    get size() {
        return this.cache.size;
    }
    keys() {
        return this.cache.keys();
    }
}
class CacheManager {
    constructor(options = {}) {
        this.maxCacheSize = options.maxCacheSize || 1000;
        this.defaultTTL = options.defaultTTL || 300000; // 5 minutes default
        this.diskCacheEnabled = options.diskCacheEnabled !== false;
        this.diskCacheDir = options.diskCacheDir || (0, path_1.join)(process.cwd(), '.spy-code', 'cache');
        this.memoryCache = new SimpleLRUCache(this.maxCacheSize);
        if (this.diskCacheEnabled) {
            this.initializeDiskCache();
        }
    }
    /**
     * Initialize disk cache directory
     */
    async initializeDiskCache() {
        try {
            await fs_1.promises.mkdir(this.diskCacheDir, { recursive: true });
        }
        catch (error) {
            console.warn(`Failed to create disk cache directory: ${error}`);
            this.diskCacheEnabled = false;
        }
    }
    /**
     * Generate cache key
     */
    generateKey(prefix, params) {
        const hash = (0, crypto_1.createHash)('sha256');
        hash.update(prefix);
        hash.update(JSON.stringify(params));
        return `${prefix}:${hash.digest('hex').substring(0, 16)}`;
    }
    /**
     * Get from cache
     */
    async get(prefix, params) {
        const key = this.generateKey(prefix, params);
        // Try memory cache first
        const memoryEntry = this.memoryCache.get(key);
        if (memoryEntry) {
            if (Date.now() - memoryEntry.timestamp < memoryEntry.ttl) {
                return memoryEntry.data;
            }
            else {
                this.memoryCache.delete(key);
            }
        }
        // Try disk cache
        if (this.diskCacheEnabled) {
            try {
                const filePath = (0, path_1.join)(this.diskCacheDir, `${key}.json`);
                const content = await fs_1.promises.readFile(filePath, 'utf-8');
                const entry = JSON.parse(content);
                if (Date.now() - entry.timestamp < entry.ttl) {
                    // Promote to memory cache
                    this.memoryCache.set(key, entry);
                    return entry.data;
                }
                else {
                    // Expired, delete from disk
                    await fs_1.promises.unlink(filePath);
                }
            }
            catch (error) {
                // File doesn't exist or can't be read, ignore
            }
        }
        return null;
    }
    /**
     * Set in cache
     */
    async set(prefix, params, data, ttl) {
        const key = this.generateKey(prefix, params);
        const entry = {
            data,
            timestamp: Date.now(),
            ttl: ttl || this.defaultTTL
        };
        // Set in memory cache
        this.memoryCache.set(key, entry);
        // Set in disk cache
        if (this.diskCacheEnabled) {
            try {
                const filePath = (0, path_1.join)(this.diskCacheDir, `${key}.json`);
                await fs_1.promises.writeFile(filePath, JSON.stringify(entry), 'utf-8');
            }
            catch (error) {
                console.warn(`Failed to write to disk cache: ${error}`);
            }
        }
    }
    /**
     * Delete from cache
     */
    async delete(prefix, params) {
        const key = this.generateKey(prefix, params);
        // Delete from memory cache
        this.memoryCache.delete(key);
        // Delete from disk cache
        if (this.diskCacheEnabled) {
            try {
                const filePath = (0, path_1.join)(this.diskCacheDir, `${key}.json`);
                await fs_1.promises.unlink(filePath);
            }
            catch (error) {
                // File doesn't exist, ignore
            }
        }
    }
    /**
     * Clear all cache
     */
    async clear() {
        // Clear memory cache
        this.memoryCache.clear();
        // Clear disk cache
        if (this.diskCacheEnabled) {
            try {
                const files = await fs_1.promises.readdir(this.diskCacheDir);
                await Promise.all(files.map(file => fs_1.promises.unlink((0, path_1.join)(this.diskCacheDir, file))));
            }
            catch (error) {
                console.warn(`Failed to clear disk cache: ${error}`);
            }
        }
    }
    /**
     * Clear cache by prefix
     */
    async clearPrefix(prefix) {
        // Clear from memory cache
        for (const key of this.memoryCache.keys()) {
            if (key.startsWith(prefix)) {
                this.memoryCache.delete(key);
            }
        }
        // Clear from disk cache
        if (this.diskCacheEnabled) {
            try {
                const files = await fs_1.promises.readdir(this.diskCacheDir);
                await Promise.all(files
                    .filter(file => file.startsWith(prefix))
                    .map(file => fs_1.promises.unlink((0, path_1.join)(this.diskCacheDir, file))));
            }
            catch (error) {
                console.warn(`Failed to clear disk cache by prefix: ${error}`);
            }
        }
    }
    /**
     * Get cache statistics
     */
    getStats() {
        return {
            memorySize: this.memoryCache.size,
            memoryUsage: this.memoryCache.size,
            diskEnabled: this.diskCacheEnabled,
            diskCacheDir: this.diskCacheDir
        };
    }
    /**
     * Clean expired entries from disk cache
     */
    async cleanExpired() {
        if (!this.diskCacheEnabled) {
            return;
        }
        try {
            const files = await fs_1.promises.readdir(this.diskCacheDir);
            const now = Date.now();
            for (const file of files) {
                try {
                    const filePath = (0, path_1.join)(this.diskCacheDir, file);
                    const content = await fs_1.promises.readFile(filePath, 'utf-8');
                    const entry = JSON.parse(content);
                    if (now - entry.timestamp >= entry.ttl) {
                        await fs_1.promises.unlink(filePath);
                    }
                }
                catch (error) {
                    // Invalid file, delete it
                    try {
                        await fs_1.promises.unlink((0, path_1.join)(this.diskCacheDir, file));
                    }
                    catch {
                        // Ignore
                    }
                }
            }
        }
        catch (error) {
            console.warn(`Failed to clean expired cache entries: ${error}`);
        }
    }
    /**
     * Update configuration
     */
    updateConfig(options) {
        if (options.maxCacheSize !== undefined) {
            this.maxCacheSize = options.maxCacheSize;
            // Recreate cache with new size
            this.memoryCache = new SimpleLRUCache(this.maxCacheSize);
        }
        if (options.defaultTTL !== undefined) {
            this.defaultTTL = options.defaultTTL;
        }
        if (options.diskCacheEnabled !== undefined) {
            this.diskCacheEnabled = options.diskCacheEnabled;
            if (this.diskCacheEnabled) {
                this.initializeDiskCache();
            }
        }
    }
}
exports.CacheManager = CacheManager;
