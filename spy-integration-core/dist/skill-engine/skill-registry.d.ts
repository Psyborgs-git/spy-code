/**
 * Skill Registry
 * Central registry for managing available skills
 */
import { SkillDefinition } from './skill-loader';
import { MatchResult } from './skill-matcher';
import { ExecutionContext, ExecutionResult } from './skill-executor';
import { MCPClient } from '../mcp-adapter/client';
export declare class SkillRegistry {
    private static instance;
    private skillLoader;
    private skillMatcher;
    private skillExecutor;
    private mcpClient;
    private initialized;
    private constructor();
    /**
     * Get singleton instance
     */
    static getInstance(): SkillRegistry;
    /**
     * Initialize the skill registry
     */
    initialize(mcpClient: MCPClient, skillsDir?: string): Promise<void>;
    /**
     * Match a request to skills
     */
    match(request: string, context?: any): MatchResult[];
    /**
     * Get the best matching skill
     */
    getBestMatch(request: string, context?: any): MatchResult | null;
    /**
     * Execute a skill
     */
    execute(skillId: string, context: ExecutionContext): Promise<ExecutionResult>;
    /**
     * Execute a skill pattern
     */
    executePattern(skillId: string, patternName: string): Promise<ExecutionResult>;
    /**
     * Get a skill by ID
     */
    getSkill(id: string): SkillDefinition | undefined;
    /**
     * Get all skills
     */
    getAllSkills(): SkillDefinition[];
    /**
     * Get skills by category
     */
    getSkillsByCategory(category: 'universal' | 'environments'): SkillDefinition[];
    /**
     * Get skills using a specific tool
     */
    getSkillsUsingTool(tool: string): SkillDefinition[];
    /**
     * Check if initialized
     */
    isInitialized(): boolean;
    /**
     * Reset the registry (useful for testing)
     */
    reset(): void;
}
/**
 * Convenience function to get the skill registry instance
 */
export declare function getSkillRegistry(): SkillRegistry;
