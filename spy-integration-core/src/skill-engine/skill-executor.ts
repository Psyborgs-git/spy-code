/**
 * Skill Executor
 * Executes skills with proper context and tool calls
 */

import { SkillDefinition } from './skill-loader';
import { MCPClient } from '../mcp-adapter/client';

export interface ExecutionContext {
  request: string;
  filePath?: string;
  nodeId?: string;
  metadata?: Record<string, any>;
}

export interface ExecutionResult {
  success: boolean;
  skill: SkillDefinition;
  output: any;
  error?: Error;
  steps: string[];
}

export class SkillExecutor {
  private mcpClient: MCPClient;
  private skills: Map<string, SkillDefinition>;

  constructor(mcpClient: MCPClient, skills: Map<string, SkillDefinition>) {
    this.mcpClient = mcpClient;
    this.skills = skills;
  }

  /**
   * Execute a skill with the given context
   */
  async execute(skillId: string, context: ExecutionContext): Promise<ExecutionResult> {
    const skill = this.skills.get(skillId);
    if (!skill) {
      throw new Error(`Skill not found: ${skillId}`);
    }

    const steps: string[] = [];
    let output: any = null;
    let error: Error | undefined;

    try {
      // Execute based on skill type
      if (skill.id.includes('navigation')) {
        output = await this.executeNavigationSkill(skill, context, steps);
      } else if (skill.id.includes('call-graph')) {
        output = await this.executeCallGraphSkill(skill, context, steps);
      } else if (skill.id.includes('semantic-search')) {
        output = await this.executeSemanticSearchSkill(skill, context, steps);
      } else if (skill.id.includes('change-tracking')) {
        output = await this.executeChangeTrackingSkill(skill, context, steps);
      } else if (skill.id.includes('graph-visualization')) {
        output = await this.executeGraphVisualizationSkill(skill, context, steps);
      } else {
        // Generic execution
        output = await this.executeGenericSkill(skill, context, steps);
      }

      return {
        success: true,
        skill,
        output,
        steps
      };
    } catch (err) {
      error = err as Error;
      return {
        success: false,
        skill,
        output: null,
        error,
        steps
      };
    }
  }

  /**
   * Execute a navigation skill
   */
  private async executeNavigationSkill(
    skill: SkillDefinition,
    context: ExecutionContext,
    steps: string[]
  ): Promise<any> {
    steps.push('Searching for relevant code...');

    // Search for the query
    const searchResults = await this.mcpClient.search(context.request);
    steps.push(`Found ${searchResults.length} results`);

    if (searchResults.length > 0) {
      steps.push('Getting node details...');
      const node = await this.mcpClient.getNode(searchResults[0].node.id);
      return { searchResults, selectedNode: node };
    }

    return { searchResults };
  }

  /**
   * Execute a call graph skill
   */
  private async executeCallGraphSkill(
    skill: SkillDefinition,
    context: ExecutionContext,
    steps: string[]
  ): Promise<any> {
    if (!context.nodeId) {
      throw new Error('Node ID required for call graph analysis');
    }

    steps.push('Getting callers...');
    const callers = await this.mcpClient.getCallers(context.nodeId, 2);

    steps.push('Getting callees...');
    const callees = await this.mcpClient.getCallees(context.nodeId, 2);

    steps.push(`Found ${callers.length} callers and ${callees.length} callees`);

    return { callers, callees };
  }

  /**
   * Execute a semantic search skill
   */
  private async executeSemanticSearchSkill(
    skill: SkillDefinition,
    context: ExecutionContext,
    steps: string[]
  ): Promise<any> {
    steps.push('Performing semantic search...');

    const results = await this.mcpClient.ask(context.request, 10);
    steps.push(`Found ${results.length} semantic matches`);

    return { results };
  }

  /**
   * Execute a change tracking skill
   */
  private async executeChangeTrackingSkill(
    skill: SkillDefinition,
    context: ExecutionContext,
    steps: string[]
  ): Promise<any> {
    const gitRef = context.metadata?.gitRef || 'HEAD~1';

    steps.push(`Finding changes since ${gitRef}...`);
    const changedNodes = await this.mcpClient.changedSince(gitRef);

    steps.push(`Found ${changedNodes.length} changed nodes`);

    return { changedNodes, gitRef };
  }

  /**
   * Execute a graph visualization skill
   */
  private async executeGraphVisualizationSkill(
    skill: SkillDefinition,
    context: ExecutionContext,
    steps: string[]
  ): Promise<any> {
    steps.push('Getting graph data...');

    const graphData = await this.mcpClient.queryGraph(`
      {
        graphData {
          nodes {
            id
            name
            kind
            filePath
          }
          edges {
            from { id }
            to { id }
            kind
          }
        }
      }
    `);

    steps.push('Graph data retrieved');

    return graphData;
  }

  /**
   * Execute a generic skill
   */
  private async executeGenericSkill(
    skill: SkillDefinition,
    context: ExecutionContext,
    steps: string[]
  ): Promise<any> {
    steps.push('Executing generic skill...');

    // Try to execute based on available tools
    if (skill.availableTools.includes('search')) {
      const results = await this.mcpClient.search(context.request);
      steps.push('Search completed');
      return { results };
    }

    steps.push('No specific execution path found');
    return { message: 'Skill executed generically' };
  }

  /**
   * Execute a skill pattern
   */
  async executePattern(
    skillId: string,
    patternName: string
  ): Promise<ExecutionResult> {
    const skill = this.skills.get(skillId);
    if (!skill) {
      throw new Error(`Skill not found: ${skillId}`);
    }

    const pattern = skill.commonPatterns.find(p => p.name === patternName);
    if (!pattern) {
      throw new Error(`Pattern not found: ${patternName}`);
    }

    const steps: string[] = [];
    const output: any = {};

    try {
      for (const step of pattern.steps) {
        steps.push(step);
        // Execute the step (this would need more sophisticated parsing)
        // For now, just track the steps
      }

      return {
        success: true,
        skill,
        output,
        steps
      };
    } catch (err) {
      return {
        success: false,
        skill,
        output: null,
        error: err as Error,
        steps
      };
    }
  }
}
