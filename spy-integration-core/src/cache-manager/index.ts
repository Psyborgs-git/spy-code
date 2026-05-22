/**
 * Cache Manager - In-memory and disk-based caching for query results
 */

import { createHash } from 'crypto';
import { promises as fs } from 'fs';
import { join } from 'path';

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

// Simple LRU cache implementation
class SimpleLRUCache<K, V> {
  private cache: Map<K, V>;
  private maxSize: number;

  constructor(maxSize: number) {
    this.cache = new Map();
    this.maxSize = maxSize;
  }

  get(key: K): V | undefined {
    const value = this.cache.get(key);
    if (value !== undefined) {
      // Move to end (most recently used)
      this.cache.delete(key);
      this.cache.set(key, value);
    }
    return value;
  }

  set(key: K, value: V): boolean {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.maxSize) {
      // Remove first item (least recently used)
      const firstKey = this.cache.keys().next().value;
      if (firstKey !== undefined) {
        this.cache.delete(firstKey);
      }
    }
    this.cache.set(key, value);
    return true;
  }

  delete(key: K): boolean {
    return this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  has(key: K): boolean {
    return this.cache.has(key);
  }

  get size(): number {
    return this.cache.size;
  }

  keys(): IterableIterator<K> {
    return this.cache.keys();
  }
}

export class CacheManager {
  private memoryCache: SimpleLRUCache<string, CacheEntry<any>>;
  private diskCacheEnabled: boolean;
  private diskCacheDir: string;
  private defaultTTL: number;
  private maxCacheSize: number;

  constructor(options: {
    maxCacheSize?: number;
    defaultTTL?: number;
    diskCacheEnabled?: boolean;
    diskCacheDir?: string;
  } = {}) {
    this.maxCacheSize = options.maxCacheSize || 1000;
    this.defaultTTL = options.defaultTTL || 300000; // 5 minutes default
    this.diskCacheEnabled = options.diskCacheEnabled !== false;
    this.diskCacheDir = options.diskCacheDir || join(process.cwd(), '.spy-code', 'cache');

    this.memoryCache = new SimpleLRUCache(this.maxCacheSize);

    if (this.diskCacheEnabled) {
      this.initializeDiskCache();
    }
  }

  /**
   * Initialize disk cache directory
   */
  private async initializeDiskCache(): Promise<void> {
    try {
      await fs.mkdir(this.diskCacheDir, { recursive: true });
    } catch (error) {
      console.warn(`Failed to create disk cache directory: ${error}`);
      this.diskCacheEnabled = false;
    }
  }

  /**
   * Generate cache key
   */
  private generateKey(prefix: string, params: any): string {
    const hash = createHash('sha256');
    hash.update(prefix);
    hash.update(JSON.stringify(params));
    return `${prefix}:${hash.digest('hex').substring(0, 16)}`;
  }

  /**
   * Get from cache
   */
  async get<T>(prefix: string, params: any): Promise<T | null> {
    const key = this.generateKey(prefix, params);

    // Try memory cache first
    const memoryEntry = this.memoryCache.get(key);
    if (memoryEntry) {
      if (Date.now() - memoryEntry.timestamp < memoryEntry.ttl) {
        return memoryEntry.data as T;
      } else {
        this.memoryCache.delete(key);
      }
    }

    // Try disk cache
    if (this.diskCacheEnabled) {
      try {
        const filePath = join(this.diskCacheDir, `${key}.json`);
        const content = await fs.readFile(filePath, 'utf-8');
        const entry: CacheEntry<T> = JSON.parse(content);

        if (Date.now() - entry.timestamp < entry.ttl) {
          // Promote to memory cache
          this.memoryCache.set(key, entry);
          return entry.data;
        } else {
          // Expired, delete from disk
          await fs.unlink(filePath);
        }
      } catch (error) {
        // File doesn't exist or can't be read, ignore
      }
    }

    return null;
  }

  /**
   * Set in cache
   */
  async set<T>(prefix: string, params: any, data: T, ttl?: number): Promise<void> {
    const key = this.generateKey(prefix, params);
    const entry: CacheEntry<T> = {
      data,
      timestamp: Date.now(),
      ttl: ttl || this.defaultTTL
    };

    // Set in memory cache
    this.memoryCache.set(key, entry);

    // Set in disk cache
    if (this.diskCacheEnabled) {
      try {
        const filePath = join(this.diskCacheDir, `${key}.json`);
        await fs.writeFile(filePath, JSON.stringify(entry), 'utf-8');
      } catch (error) {
        console.warn(`Failed to write to disk cache: ${error}`);
      }
    }
  }

  /**
   * Delete from cache
   */
  async delete(prefix: string, params: any): Promise<void> {
    const key = this.generateKey(prefix, params);

    // Delete from memory cache
    this.memoryCache.delete(key);

    // Delete from disk cache
    if (this.diskCacheEnabled) {
      try {
        const filePath = join(this.diskCacheDir, `${key}.json`);
        await fs.unlink(filePath);
      } catch (error) {
        // File doesn't exist, ignore
      }
    }
  }

  /**
   * Clear all cache
   */
  async clear(): Promise<void> {
    // Clear memory cache
    this.memoryCache.clear();

    // Clear disk cache
    if (this.diskCacheEnabled) {
      try {
        const files = await fs.readdir(this.diskCacheDir);
        await Promise.all(
          files.map(file => fs.unlink(join(this.diskCacheDir, file)))
        );
      } catch (error) {
        console.warn(`Failed to clear disk cache: ${error}`);
      }
    }
  }

  /**
   * Clear cache by prefix
   */
  async clearPrefix(prefix: string): Promise<void> {
    // Clear from memory cache
    for (const key of this.memoryCache.keys()) {
      if (key.startsWith(prefix)) {
        this.memoryCache.delete(key);
      }
    }

    // Clear from disk cache
    if (this.diskCacheEnabled) {
      try {
        const files = await fs.readdir(this.diskCacheDir);
        await Promise.all(
          files
            .filter(file => file.startsWith(prefix))
            .map(file => fs.unlink(join(this.diskCacheDir, file)))
        );
      } catch (error) {
        console.warn(`Failed to clear disk cache by prefix: ${error}`);
      }
    }
  }

  /**
   * Get cache statistics
   */
  getStats(): {
    memorySize: number;
    memoryUsage: number;
    diskEnabled: boolean;
    diskCacheDir: string;
  } {
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
  async cleanExpired(): Promise<void> {
    if (!this.diskCacheEnabled) {
      return;
    }

    try {
      const files = await fs.readdir(this.diskCacheDir);
      const now = Date.now();

      for (const file of files) {
        try {
          const filePath = join(this.diskCacheDir, file);
          const content = await fs.readFile(filePath, 'utf-8');
          const entry: CacheEntry<any> = JSON.parse(content);

          if (now - entry.timestamp >= entry.ttl) {
            await fs.unlink(filePath);
          }
        } catch (error) {
          // Invalid file, delete it
          try {
            await fs.unlink(join(this.diskCacheDir, file));
          } catch {
            // Ignore
          }
        }
      }
    } catch (error) {
      console.warn(`Failed to clean expired cache entries: ${error}`);
    }
  }

  /**
   * Update configuration
   */
  updateConfig(options: {
    maxCacheSize?: number;
    defaultTTL?: number;
    diskCacheEnabled?: boolean;
  }): void {
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
