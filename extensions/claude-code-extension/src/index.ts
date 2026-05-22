/**
 * Claude Code Extension Main Entry Point
 * This extension integrates spy-code with Claude Code via MCP
 */

import { ClaudeCodeAdapter } from '@spy-code/claude-code-adapter';
import { MCPClient, CacheManager, eventBus, EventType } from '@spy-code/integration-core';
import * as fs from 'fs/promises';
import * as path from 'path';

let adapter: ClaudeCodeAdapter;
let mcpClient: MCPClient;
let cacheManager: CacheManager;

export async function activate(context: any): Promise<void> {
  console.log('Spy-Code Claude Code extension is activating');

  // Initialize adapter
  adapter = new ClaudeCodeAdapter({
    spyCodePath: 'spy-code',
    dbPath: '.spy-code/graph.db',
    enableMCP: true,
    enableHooks: true,
    cacheEnabled: true,
    cacheTTL: 300000,
    maxCacheSize: 1000
  });

  await adapter.initialize();
  await adapter.activate();

  // Initialize MCP client
  try {
    mcpClient = new MCPClient();
    await mcpClient.connect();
    await mcpClient.initialize();
    eventBus.emit(EventType.MCP_CONNECTED, {});
    console.log('MCP client connected');
  } catch (error) {
    console.error('Failed to initialize MCP client:', error);
  }

  // Initialize cache manager
  cacheManager = new CacheManager({
    maxCacheSize: 1000,
    defaultTTL: 300000,
    diskCacheEnabled: true
  });

  // Setup MCP configuration
  await setupMCPConfig();

  // Register skills
  await registerSkills();

  // Register workflows
  await registerWorkflows();

  console.log('Spy-Code Claude Code extension activated successfully');
}

export async function deactivate(): Promise<void> {
  console.log('Spy-Code Claude Code extension is deactivating');

  if (adapter) {
    await adapter.deactivate();
  }

  if (mcpClient) {
    await mcpClient.disconnect();
  }

  if (cacheManager) {
    await cacheManager.clear();
  }

  console.log('Spy-Code Claude Code extension deactivated');
}

/**
 * Setup MCP configuration for Claude Code
 */
async function setupMCPConfig(): Promise<void> {
  const claudeDir = path.join(process.cwd(), '.claude');
  const mcpConfigPath = path.join(claudeDir, 'mcp.json');

  try {
    await fs.mkdir(claudeDir, { recursive: true });

    const mcpConfig = {
      mcpServers: {
        'spy-code': {
          command: 'spy-code',
          args: ['serve', '--mcp'],
          env: {
            SPY_DB_PATH: '.spy-code/graph.db'
          }
        }
      }
    };

    await fs.writeFile(mcpConfigPath, JSON.stringify(mcpConfig, null, 2));
    console.log('MCP configuration written to', mcpConfigPath);
  } catch (error) {
    console.error('Failed to setup MCP config:', error);
  }
}

/**
 * Register Claude Code skills
 */
async function registerSkills(): Promise<void> {
  const skillsDir = path.join(process.cwd(), '.claude', 'skills');

  try {
    await fs.mkdir(skillsDir, { recursive: true });

    // Search skill
    const searchSkill = {
      name: 'spy-code-search',
      description: 'Search the codebase using spy-code',
      parameters: {
        type: 'object',
        properties: {
          query: { type: 'string', description: 'Search query' },
          kind: { type: 'string', description: 'Node kind filter (function, class, constant)' }
        },
        required: ['query']
      }
    };
    await fs.writeFile(
      path.join(skillsDir, 'spy-code-search.json'),
      JSON.stringify(searchSkill, null, 2)
    );

    // Semantic search skill
    const semanticSearchSkill = {
      name: 'spy-code-semantic-search',
      description: 'Semantic search using embeddings',
      parameters: {
        type: 'object',
        properties: {
          query: { type: 'string', description: 'Natural language query' }
        },
        required: ['query']
      }
    };
    await fs.writeFile(
      path.join(skillsDir, 'spy-code-semantic-search.json'),
      JSON.stringify(semanticSearchSkill, null, 2)
    );

    // Get node skill
    const getNodeSkill = {
      name: 'spy-code-get-node',
      description: 'Get detailed information about a node',
      parameters: {
        type: 'object',
        properties: {
          nodeId: { type: 'string', description: 'Node ID' }
        },
        required: ['nodeId']
      }
    };
    await fs.writeFile(
      path.join(skillsDir, 'spy-code-get-node.json'),
      JSON.stringify(getNodeSkill, null, 2)
    );

    // Callers skill
    const callersSkill = {
      name: 'spy-code-callers',
      description: 'Find callers of a function',
      parameters: {
        type: 'object',
        properties: {
          nodeId: { type: 'string', description: 'Node ID' },
          depth: { type: 'number', description: 'Call depth (default: 1)' }
        },
        required: ['nodeId']
      }
    };
    await fs.writeFile(
      path.join(skillsDir, 'spy-code-callers.json'),
      JSON.stringify(callersSkill, null, 2)
    );

    // Callees skill
    const calleesSkill = {
      name: 'spy-code-callees',
      description: 'Find callees of a function',
      parameters: {
        type: 'object',
        properties: {
          nodeId: { type: 'string', description: 'Node ID' },
          depth: { type: 'number', description: 'Call depth (default: 1)' }
        },
        required: ['nodeId']
      }
    };
    await fs.writeFile(
      path.join(skillsDir, 'spy-code-callees.json'),
      JSON.stringify(calleesSkill, null, 2)
    );

    console.log('Claude Code skills registered');
  } catch (error) {
    console.error('Failed to register skills:', error);
  }
}

/**
 * Register Claude Code workflows
 */
async function registerWorkflows(): Promise<void> {
  const workflowsDir = path.join(process.cwd(), '.claude', 'workflows');

  try {
    await fs.mkdir(workflowsDir, { recursive: true });

    // Analyze function workflow
    const analyzeFunctionWorkflow = `---
description: Analyze a function and its relationships
---

1. Use spy-code-get-node to get function details
2. Use spy-code-callers to find all callers (depth: 2)
3. Use spy-code-callees to find all callees (depth: 2)
4. Summarize the function's purpose and relationships
5. Identify any potential issues or improvements
`;
    await fs.writeFile(
      path.join(workflowsDir, 'analyze-function.md'),
      analyzeFunctionWorkflow
    );

    // Impact analysis workflow
    const impactAnalysisWorkflow = `---
description: Analyze the impact of changing a function
---

1. Use spy-code-get-node to get function details
2. Use spy-code-callers to find all callers recursively (depth: 5)
3. Identify all affected files from the callers
4. Check for any critical callers (e.g., tests, main entry points)
5. Generate an impact report with:
   - List of affected files
   - Risk assessment
   - Recommended testing approach
`;
    await fs.writeFile(
      path.join(workflowsDir, 'impact-analysis.md'),
      impactAnalysisWorkflow
    );

    // Code review workflow
    const codeReviewWorkflow = `---
description: Review code using spy-code context
---

1. Use spy-code-search to find related functions in the same file
2. Use spy-code-semantic-search to find similar patterns in the codebase
3. Check for:
   - Duplicate code
   - Inconsistent patterns
   - Missing error handling
   - Performance issues
4. Generate review comments with suggestions
`;
    await fs.writeFile(
      path.join(workflowsDir, 'code-review.md'),
      codeReviewWorkflow
    );

    console.log('Claude Code workflows registered');
  } catch (error) {
    console.error('Failed to register workflows:', error);
  }
}
