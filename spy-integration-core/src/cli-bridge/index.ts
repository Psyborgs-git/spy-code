/**
 * CLI Bridge - Wrapper around spy-code CLI commands
 */

import { spawn } from 'child_process';
import {
  Node,
  SearchResult,
  Edge,
  IndexStats,
  GraphData,
  GraphFilter,
  SearchOptions,
  SemanticSearchOptions,
  CLIConfig,
  CLIError
} from '../types';

export class CLIBridge {
  private config: CLIConfig;

  constructor(config: Partial<CLIConfig> = {}) {
    this.config = {
      command: config.command || 'spy-code',
      args: config.args || [],
      timeout: config.timeout || 30000,
      env: config.env || process.env as Record<string, string>
    };
  }

  /**
   * Execute a spy-code CLI command
   */
  private async executeCommand(
    args: string[],
    options: { timeout?: number; cwd?: string } = {}
  ): Promise<string> {
    const { timeout = this.config.timeout, cwd } = options;
    const fullArgs = [...this.config.args, ...args];

    return new Promise((resolve, reject) => {
      const child = spawn(this.config.command, fullArgs, {
        env: this.config.env,
        cwd: cwd || process.cwd()
      });

      let stdout = '';

      child.stdout?.on('data', (data) => {
        stdout += data.toString();
      });

      child.stderr?.on('data', () => {
        // Ignore stderr for now
      });

      const timer = setTimeout(() => {
        child.kill();
        reject(new CLIError(`Command timed out after ${timeout}ms`));
      }, timeout);

      child.on('close', (code) => {
        clearTimeout(timer);
        if (code === 0) {
          resolve(stdout);
        } else {
          reject(new CLIError(`Command failed with exit code ${code}`));
        }
      });

      child.on('error', (error) => {
        clearTimeout(timer);
        reject(new CLIError(`Failed to spawn command: ${error.message}`, { error }));
      });
    });
  }

  /**
   * Search for nodes by name/description
   */
  async search(query: string, options: SearchOptions = {}): Promise<SearchResult[]> {
    const args = ['search', query];

    if (options.kind) {
      args.push('--kind', options.kind);
    }

    args.push('--json');

    try {
      const output = await this.executeCommand(args);
      const data = JSON.parse(output);
      return data.results || [];
    } catch (error) {
      throw new CLIError(`Search failed: ${error}`, { error });
    }
  }

  /**
   * Semantic search using embeddings
   */
  async semanticSearch(query: string, options: SemanticSearchOptions = {}): Promise<SearchResult[]> {
    const args = ['search', query, '--semantic', '--json'];

    if (options.limit) {
      args.push('--limit', options.limit.toString());
    }

    try {
      const output = await this.executeCommand(args);
      const data = JSON.parse(output);
      return data.results || [];
    } catch (error) {
      throw new CLIError(`Semantic search failed: ${error}`, { error });
    }
  }

  /**
   * Get a specific node by ID
   */
  async getNode(nodeId: string): Promise<Node | null> {
    try {
      const output = await this.executeCommand(['get', nodeId, '--json']);
      const data = JSON.parse(output);
      return data.node || null;
    } catch (error) {
      if (error instanceof CLIError && error.message.includes('not found')) {
        return null;
      }
      throw new CLIError(`Get node failed: ${error}`, { error });
    }
  }

  /**
   * Get callers of a node
   */
  async getCallers(nodeId: string, depth: number = 1): Promise<Edge[]> {
    try {
      const output = await this.executeCommand(['callers', nodeId, '--depth', depth.toString(), '--json']);
      const data = JSON.parse(output);
      return data.edges || [];
    } catch (error) {
      throw new CLIError(`Get callers failed: ${error}`, { error });
    }
  }

  /**
   * Get callees of a node
   */
  async getCallees(nodeId: string, depth: number = 1): Promise<Edge[]> {
    try {
      const output = await this.executeCommand(['callees', nodeId, '--depth', depth.toString(), '--json']);
      const data = JSON.parse(output);
      return data.edges || [];
    } catch (error) {
      throw new CLIError(`Get callees failed: ${error}`, { error });
    }
  }

  /**
   * Get graph data with optional filters
   */
  async getGraphData(filter?: GraphFilter): Promise<GraphData> {
    const args = ['query', '--json'];

    let query = '{ graphData { nodes { id kind name language filePath startLine endLine } edges { fromId toId kind confidence } } }';

    if (filter) {
      // Build filter query
      const filters: string[] = [];
      if (filter.filePath) filters.push(`filePath: "${filter.filePath}"`);
      if (filter.nodeKinds?.length) filters.push(`nodeKinds: [${filter.nodeKinds.map(k => `"${k}"`).join(', ')}]`);
      if (filter.languages?.length) filters.push(`languages: [${filter.languages.map(l => `"${l}"`).join(', ')}]`);
      if (filter.edgeKinds?.length) filters.push(`edgeKinds: [${filter.edgeKinds.map(k => `"${k}"`).join(', ')}]`);

      if (filters.length) {
        query = `{ graphData(filter: { ${filters.join(', ')} }) { nodes { id kind name language filePath startLine endLine } edges { fromId toId kind confidence } } }`;
      }
    }

    args.push(query);

    try {
      const output = await this.executeCommand(args);
      const data = JSON.parse(output);
      return data.data?.graphData || { nodes: [], edges: [] };
    } catch (error) {
      throw new CLIError(`Get graph data failed: ${error}`, { error });
    }
  }

  /**
   * Get index statistics
   */
  async getStats(): Promise<IndexStats> {
    try {
      const output = await this.executeCommand(['stats', '--json']);
      const data = JSON.parse(output);
      return data.stats || { nodeCount: 0, edgeCount: 0, fileCount: 0 };
    } catch (error) {
      throw new CLIError(`Get stats failed: ${error}`, { error });
    }
  }

  /**
   * Reindex the codebase
   */
  async reindex(full: boolean = false): Promise<IndexStats> {
    const args = ['index'];
    if (full) {
      args.push('--full');
    }
    args.push('--json');

    try {
      const output = await this.executeCommand(args);
      const data = JSON.parse(output);
      return data.stats || { nodeCount: 0, edgeCount: 0, fileCount: 0 };
    } catch (error) {
      throw new CLIError(`Reindex failed: ${error}`, { error });
    }
  }

  /**
   * Get nodes changed since a git ref
   */
  async changedSince(ref: string): Promise<Node[]> {
    try {
      const output = await this.executeCommand(['changed', ref, '--json']);
      const data = JSON.parse(output);
      return data.nodes || [];
    } catch (error) {
      throw new CLIError(`Changed since failed: ${error}`, { error });
    }
  }

  /**
   * Generate embeddings for semantic search
   */
  async generateEmbeddings(full: boolean = false): Promise<void> {
    const args = ['embed'];
    if (full) {
      args.push('--full');
    }

    try {
      await this.executeCommand(args);
    } catch (error) {
      throw new CLIError(`Generate embeddings failed: ${error}`, { error });
    }
  }

  /**
   * Ask a natural language question
   */
  async ask(query: string, limit: number = 20): Promise<SearchResult[]> {
    const args = ['ask', query, '--limit', limit.toString(), '--json'];

    try {
      const output = await this.executeCommand(args);
      const data = JSON.parse(output);
      return data.results || [];
    } catch (error) {
      throw new CLIError(`Ask failed: ${error}`, { error });
    }
  }

  /**
   * Initialize spy-code in a directory
   */
  async init(cwd?: string): Promise<void> {
    try {
      await this.executeCommand(['init'], { cwd });
    } catch (error) {
      throw new CLIError(`Init failed: ${error}`, { error });
    }
  }

  /**
   * Check if spy-code is available
   */
  async isAvailable(): Promise<boolean> {
    try {
      await this.executeCommand(['--version']);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Update CLI configuration
   */
  updateConfig(config: Partial<CLIConfig>): void {
    this.config = { ...this.config, ...config };
  }
}
