/**
 * MCP Adapter - Model Context Protocol client implementation
 */

import { spawn, ChildProcess } from 'child_process';
import { EventEmitter } from 'events';
import {
  Node,
  SearchResult,
  Edge,
  IndexStats,
  SearchOptions,
  MCPError
} from '../types';

interface MCPRequest {
  jsonrpc: string;
  id: number | string;
  method: string;
  params?: any;
}

interface MCPResponse {
  jsonrpc: string;
  id: number | string;
  result?: any;
  error?: {
    code: number;
    message: string;
  };
}

interface MCPTool {
  name: string;
  description: string;
  inputSchema: any;
}

interface MCPResource {
  uri: string;
  name: string;
  description: string;
  mimeType: string;
}

export class MCPClient extends EventEmitter {
  private process: ChildProcess | null = null;
  private requestId: number = 0;
  private pendingRequests: Map<number | string, {
    resolve: (value: any) => void;
    reject: (error: Error) => void;
  }> = new Map();
  private isConnected: boolean = false;
  private configPath: string;

  constructor(configPath: string = 'spy.config.json') {
    super();
    this.configPath = configPath;
  }

  /**
   * Connect to MCP server
   */
  async connect(): Promise<void> {
    if (this.isConnected) {
      return;
    }

    return new Promise((resolve, reject) => {
      this.process = spawn('spy-code', ['serve', '--mcp'], {
        env: process.env
      });

      let stdout = '';

      this.process.stdout?.on('data', (data) => {
        stdout += data.toString();
        this.processMessages(stdout);
        stdout = '';
      });

      this.process.stderr?.on('data', (data) => {
        // Log stderr for debugging
        console.error('MCP stderr:', data.toString());
      });

      this.process.on('error', (error) => {
        reject(new MCPError(`Failed to start MCP server: ${error.message}`, { error }));
      });

      this.process.on('close', (code) => {
        this.isConnected = false;
        this.emit('close', code);
        if (code !== 0 && this.pendingRequests.size > 0) {
          const error = new MCPError(`MCP server closed with code ${code}`);
          this.pendingRequests.forEach(({ reject }) => reject(error));
          this.pendingRequests.clear();
        }
      });

      // Wait for initialization
      setTimeout(() => {
        this.isConnected = true;
        this.emit('connected');
        resolve();
      }, 500);
    });
  }

  /**
   * Process incoming messages from MCP server
   */
  private processMessages(data: string): void {
    const lines = data.trim().split('\n');

    for (const line of lines) {
      if (!line.trim()) continue;

      try {
        const message: MCPResponse = JSON.parse(line);
        this.handleResponse(message);
      } catch (error) {
        this.emit('error', new MCPError(`Failed to parse MCP message: ${error}`, { error, line }));
      }
    }
  }

  /**
   * Handle MCP response
   */
  private handleResponse(response: MCPResponse): void {
    const pending = this.pendingRequests.get(response.id);

    if (!pending) {
      return;
    }

    this.pendingRequests.delete(response.id);

    if (response.error) {
      pending.reject(new MCPError(response.error.message, response.error));
    } else {
      pending.resolve(response.result);
    }
  }

  /**
   * Send request to MCP server
   */
  private async sendRequest(method: string, params?: any): Promise<any> {
    if (!this.isConnected) {
      throw new MCPError('MCP client is not connected');
    }

    const id = ++this.requestId;
    const request: MCPRequest = {
      jsonrpc: '2.0',
      id,
      method,
      params
    };

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });

      const message = JSON.stringify(request) + '\n';
      this.process?.stdin?.write(message);

      // Timeout after 30 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new MCPError(`Request timeout: ${method}`));
        }
      }, 30000);
    });
  }

  /**
   * Initialize MCP session
   */
  async initialize(): Promise<void> {
    await this.sendRequest('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {
        tools: {},
        resources: {}
      },
      clientInfo: {
        name: 'spy-code-integration',
        version: '0.1.0'
      }
    });

    await this.sendRequest('initialized', {});
  }

  /**
   * List available tools
   */
  async listTools(): Promise<MCPTool[]> {
    const response = await this.sendRequest('tools/list');
    return response.tools || [];
  }

  /**
   * Call a tool
   */
  async callTool(name: string, args: any): Promise<any> {
    const response = await this.sendRequest('tools/call', {
      name,
      arguments: args
    });

    // Parse content from response
    if (response.content && Array.isArray(response.content)) {
      const textContent = response.content.find((c: any) => c.type === 'text');
      if (textContent) {
        try {
          return JSON.parse(textContent.text);
        } catch {
          return textContent.text;
        }
      }
    }

    return response;
  }

  /**
   * List available resources
   */
  async listResources(): Promise<MCPResource[]> {
    const response = await this.sendRequest('resources/list');
    return response.resources || [];
  }

  /**
   * Read a resource
   */
  async readResource(uri: string): Promise<any> {
    const response = await this.sendRequest('resources/read', {
      uri
    });

    if (response.contents && Array.isArray(response.contents)) {
      const content = response.contents[0];
      if (content.mimeType === 'application/json') {
        try {
          return JSON.parse(content.text);
        } catch {
          return content.text;
        }
      }
      return content.text;
    }

    return response;
  }

  /**
   * Search nodes using MCP
   */
  async search(query: string, options: SearchOptions = {}): Promise<SearchResult[]> {
    const result = await this.callTool('search', {
      query,
      kind: options.kind,
      limit: options.limit
    });

    return result || [];
  }

  /**
   * Get node using MCP
   */
  async getNode(nodeId: string): Promise<Node | null> {
    const result = await this.callTool('get_node', {
      node_id: nodeId
    });

    try {
      return JSON.parse(result);
    } catch {
      return null;
    }
  }

  /**
   * Get callers using MCP
   */
  async getCallers(nodeId: string, depth: number = 1): Promise<Edge[]> {
    const result = await this.callTool('find_callers', {
      node_id: nodeId,
      depth
    });

    try {
      return JSON.parse(result);
    } catch {
      return [];
    }
  }

  /**
   * Get callees using MCP
   */
  async getCallees(nodeId: string, depth: number = 1): Promise<Edge[]> {
    const result = await this.callTool('find_callees', {
      node_id: nodeId,
      depth
    });

    try {
      return JSON.parse(result);
    } catch {
      return [];
    }
  }

  /**
   * Get changed nodes using MCP
   */
  async changedSince(ref: string): Promise<Node[]> {
    const result = await this.callTool('changed_since', {
      git_ref: ref
    });

    try {
      return JSON.parse(result);
    } catch {
      return [];
    }
  }

  /**
   * Get stats using MCP
   */
  async getStats(): Promise<IndexStats> {
    const result = await this.callTool('stats', {});

    try {
      return JSON.parse(result);
    } catch {
      return { nodeCount: 0, edgeCount: 0, fileCount: 0 };
    }
  }

  /**
   * Run GraphQL query using MCP
   */
  async queryGraph(query: string, variables?: any): Promise<any> {
    const result = await this.callTool('query_graph', {
      query,
      variables
    });

    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }

  /**
   * Generate embeddings using MCP
   */
  async generateEmbeddings(full: boolean = false): Promise<void> {
    await this.callTool('embed', { full });
  }

  /**
   * Ask natural language question using MCP
   */
  async ask(query: string, limit: number = 20): Promise<SearchResult[]> {
    const result = await this.callTool('ask', {
      query,
      limit
    });

    try {
      return JSON.parse(result);
    } catch {
      return [];
    }
  }

  /**
   * Disconnect from MCP server
   */
  async disconnect(): Promise<void> {
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
    this.isConnected = false;
    this.pendingRequests.clear();
    this.emit('disconnected');
  }

  /**
   * Check if connected
   */
  connected(): boolean {
    return this.isConnected;
  }
}
