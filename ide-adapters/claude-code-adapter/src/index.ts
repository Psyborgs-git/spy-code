/**
 * Claude Code Adapter - Adapter for Claude Code extension API
 * Claude Code has built-in MCP support and workflow system
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import {
  IDEAdapter,
  IDEConfig,
  NotificationType,
  CursorPosition,
  Selection,
  Reference
} from '@spy-code/integration-core';

const execAsync = promisify(exec);

export class ClaudeCodeAdapter implements IDEAdapter {
  private config: IDEConfig;
  private claudeCodePath: string;
  private workspacePath: string;

  constructor(config: Partial<IDEConfig> = {}) {
    this.config = {
      spyCodePath: config.spyCodePath || 'spy-code',
      dbPath: config.dbPath || '.spy-code/graph.db',
      enableMCP: config.enableMCP !== false,
      enableHooks: config.enableHooks !== false,
      cacheEnabled: config.cacheEnabled !== false,
      cacheTTL: config.cacheTTL || 300000,
      maxCacheSize: config.maxCacheSize || 1000
    };
    
    this.claudeCodePath = 'claude-code';
    this.workspacePath = process.cwd();
  }

  /**
   * Initialize the adapter
   */
  async initialize(): Promise<void> {
    console.log('Initializing Claude Code adapter');
    
    // Check if Claude Code is available
    try {
      await execAsync(`${this.claudeCodePath} --version`);
    } catch (error) {
      console.warn('Claude Code not found in PATH');
    }
    
    // Setup MCP configuration for Claude Code
    await this.setupMCPConfig();
    
    console.log('Claude Code adapter initialized');
  }

  /**
   * Activate the adapter
   */
  async activate(): Promise<void> {
    console.log('Activating Claude Code adapter');
    // Register skills and workflows
    await this.registerSkills();
    await this.registerWorkflows();
  }

  /**
   * Deactivate the adapter
   */
  async deactivate(): Promise<void> {
    console.log('Deactivating Claude Code adapter');
  }

  /**
   * Show the spy-code panel (not applicable in Claude Code)
   */
  showPanel(): void {
    console.log('Panel not applicable in Claude Code');
  }

  /**
   * Hide the spy-code panel (not applicable in Claude Code)
   */
  hidePanel(): void {
    console.log('Panel not applicable in Claude Code');
  }

  /**
   * Show a notification
   */
  showNotification(message: string, type: NotificationType): void {
    // Claude Code uses stdout/stderr for notifications
    const prefix = type === NotificationType.ERROR ? '[ERROR]' : 
                   type === NotificationType.WARNING ? '[WARNING]' : '[INFO]';
    console.log(`${prefix} ${message}`);
  }

  /**
   * Get the current file
   */
  getCurrentFile(): string | null {
    // Claude Code tracks the current file in its context
    // This would need to be retrieved from Claude Code's state
    return null;
  }

  /**
   * Get the current selection
   */
  getCurrentSelection(): Selection | null {
    // Claude Code tracks selection in its context
    return null;
  }

  /**
   * Go to definition at a specific location
   */
  async goToDefinition(filePath: string, line: number): Promise<void> {
    // Claude Code can open files at specific lines
    try {
      await execAsync(`${this.claudeCodePath} open ${filePath}:${line}`);
    } catch (error) {
      console.error(`Failed to open file: ${error}`);
    }
  }

  /**
   * Show references
   */
  async showReferences(references: Reference[]): Promise<void> {
    console.log(`Found ${references.length} references:`);
    for (const ref of references) {
      console.log(`  - ${ref.node.name} at ${ref.filePath}:${ref.line}`);
    }
  }

  /**
   * Get the current configuration
   */
  getConfig(): IDEConfig {
    return { ...this.config };
  }

  /**
   * Update the configuration
   */
  updateConfig(config: Partial<IDEConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Setup MCP configuration for Claude Code
   */
  private async setupMCPConfig(): Promise<void> {
    // Claude Code uses MCP configuration in .claude/mcp.json
    const mcpConfig = {
      mcpServers: {
        'spy-code': {
          command: this.config.spyCodePath,
          args: ['serve', '--mcp'],
          env: {
            SPY_DB_PATH: this.config.dbPath
          }
        }
      }
    };

    // Write MCP configuration
    // This would typically be done by the user or during extension installation
    console.log('MCP configuration for spy-code:', JSON.stringify(mcpConfig, null, 2));
  }

  /**
   * Register Claude Code skills
   */
  private async registerSkills(): Promise<void> {
    // Claude Code skills are defined in .claude/skills/
    // This would create skill files for spy-code operations
    
    const skills = [
      {
        name: 'spy-code-search',
        description: 'Search the codebase using spy-code',
        parameters: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'Search query' },
            kind: { type: 'string', description: 'Node kind filter (function, class, constant)' }
          }
        }
      },
      {
        name: 'spy-code-semantic-search',
        description: 'Semantic search using embeddings',
        parameters: {
          type: 'object',
          properties: {
            query: { type: 'string', description: 'Natural language query' }
          }
        }
      },
      {
        name: 'spy-code-get-node',
        description: 'Get detailed information about a node',
        parameters: {
          type: 'object',
          properties: {
            nodeId: { type: 'string', description: 'Node ID' }
          }
        }
      },
      {
        name: 'spy-code-callers',
        description: 'Find callers of a function',
        parameters: {
          type: 'object',
          properties: {
            nodeId: { type: 'string', description: 'Node ID' },
            depth: { type: 'number', description: 'Call depth' }
          }
        }
      },
      {
        name: 'spy-code-callees',
        description: 'Find callees of a function',
        parameters: {
          type: 'object',
          properties: {
            nodeId: { type: 'string', description: 'Node ID' },
            depth: { type: 'number', description: 'Call depth' }
          }
        }
      }
    ];

    console.log('Registered Claude Code skills:', skills.map(s => s.name));
  }

  /**
   * Register Claude Code workflows
   */
  private async registerWorkflows(): Promise<void> {
    // Claude Code workflows are defined in .claude/workflows/
    // This would create workflow templates for common tasks
    
    const workflows = [
      {
        name: 'analyze-function',
        description: 'Analyze a function and its relationships',
        steps: [
          'Get function details',
          'Find callers',
          'Find callees',
          'Generate summary'
        ]
      },
      {
        name: 'impact-analysis',
        description: 'Analyze the impact of changing a function',
        steps: [
          'Get function details',
          'Find all callers recursively',
          'Identify affected files',
          'Generate impact report'
        ]
      },
      {
        name: 'code-review',
        description: 'Review code using spy-code context',
        steps: [
          'Get relevant nodes',
          'Check for patterns',
          'Generate review comments'
        ]
      }
    ];

    console.log('Registered Claude Code workflows:', workflows.map(w => w.name));
  }

  /**
   * Get workspace path
   */
  getWorkspacePath(): string {
    return this.workspacePath;
  }

  /**
   * Set workspace path
   */
  setWorkspacePath(path: string): void {
    this.workspacePath = path;
  }

  /**
   * Execute a Claude Code command
   */
  async executeCommand(command: string, args: string[]): Promise<string> {
    try {
      const { stdout } = await execAsync(`${this.claudeCodePath} ${command} ${args.join(' ')}`);
      return stdout;
    } catch (error) {
      throw new Error(`Claude Code command failed: ${error}`);
    }
  }

  /**
   * Get Claude Code version
   */
  async getVersion(): Promise<string> {
    try {
      const { stdout } = await execAsync(`${this.claudeCodePath} --version`);
      return stdout.trim();
    } catch {
      return 'unknown';
    }
  }
}
