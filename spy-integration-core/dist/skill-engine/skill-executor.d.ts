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
export declare class SkillExecutor {
    private mcpClient;
    private skills;
    constructor(mcpClient: MCPClient, skills: Map<string, SkillDefinition>);
    /**
     * Execute a skill with the given context
     */
    execute(skillId: string, context: ExecutionContext): Promise<ExecutionResult>;
    /**
     * Execute a navigation skill
     */
    private executeNavigationSkill;
    /**
     * Execute a call graph skill
     */
    private executeCallGraphSkill;
    /**
     * Execute a semantic search skill
     */
    private executeSemanticSearchSkill;
    /**
     * Execute a change tracking skill
     */
    private executeChangeTrackingSkill;
    /**
     * Execute a graph visualization skill
     */
    private executeGraphVisualizationSkill;
    /**
     * Execute a generic skill
     */
    private executeGenericSkill;
    /**
     * Execute a skill pattern
     */
    executePattern(skillId: string, patternName: string): Promise<ExecutionResult>;
}
