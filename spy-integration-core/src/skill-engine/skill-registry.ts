/**
 * Skill Registry
 * Central registry for managing available skills
 */

import { SkillLoader, SkillDefinition } from './skill-loader';
import { SkillMatcher, MatchResult } from './skill-matcher';
import { SkillExecutor, ExecutionContext, ExecutionResult } from './skill-executor';
import { MCPClient } from '../mcp-adapter/client';

export class SkillRegistry {
  private static instance: SkillRegistry;

  private skillLoader: SkillLoader;
  private skillMatcher: SkillMatcher | null = null;
  private skillExecutor: SkillExecutor | null = null;
  private mcpClient: MCPClient | null = null;
  private initialized: boolean = false;

  private constructor() {
    this.skillLoader = new SkillLoader();
  }

  /**
   * Get singleton instance
   */
  static getInstance(): SkillRegistry {
    if (!SkillRegistry.instance) {
      SkillRegistry.instance = new SkillRegistry();
    }
    return SkillRegistry.instance;
  }

  /**
   * Initialize the skill registry
   */
  async initialize(mcpClient: MCPClient, skillsDir?: string): Promise<void> {
    if (this.initialized) {
      return;
    }

    this.mcpClient = mcpClient;

    if (skillsDir) {
      this.skillLoader = new SkillLoader(skillsDir);
    }

    // Load all skills
    await this.skillLoader.loadAll();

    // Initialize matcher and executor
    const skills = this.skillLoader.getAllSkills();
    const skillsMap = new Map(skills.map(s => [s.id, s]));

    this.skillMatcher = new SkillMatcher(skillsMap);
    this.skillExecutor = new SkillExecutor(mcpClient, skillsMap);

    this.initialized = true;
  }

  /**
   * Match a request to skills
   */
  match(request: string, context?: any): MatchResult[] {
    if (!this.skillMatcher) {
      throw new Error('Skill registry not initialized');
    }
    return this.skillMatcher.match(request, context);
  }

  /**
   * Get the best matching skill
   */
  getBestMatch(request: string, context?: any): MatchResult | null {
    if (!this.skillMatcher) {
      throw new Error('Skill registry not initialized');
    }
    return this.skillMatcher.getBestMatch(request, context);
  }

  /**
   * Execute a skill
   */
  async execute(skillId: string, context: ExecutionContext): Promise<ExecutionResult> {
    if (!this.skillExecutor) {
      throw new Error('Skill registry not initialized');
    }
    return this.skillExecutor.execute(skillId, context);
  }

  /**
   * Execute a skill pattern
   */
  async executePattern(
    skillId: string,
    patternName: string
  ): Promise<ExecutionResult> {
    if (!this.skillExecutor) {
      throw new Error('Skill registry not initialized');
    }
    return this.skillExecutor.executePattern(skillId, patternName);
  }

  /**
   * Get a skill by ID
   */
  getSkill(id: string): SkillDefinition | undefined {
    return this.skillLoader.getSkill(id);
  }

  /**
   * Get all skills
   */
  getAllSkills(): SkillDefinition[] {
    return this.skillLoader.getAllSkills();
  }

  /**
   * Get skills by category
   */
  getSkillsByCategory(category: 'universal' | 'environments'): SkillDefinition[] {
    return this.skillLoader.getSkillsByCategory(category);
  }

  /**
   * Get skills using a specific tool
   */
  getSkillsUsingTool(tool: string): SkillDefinition[] {
    if (!this.skillMatcher) {
      throw new Error('Skill registry not initialized');
    }
    return this.skillMatcher.getSkillsUsingTool(tool);
  }

  /**
   * Check if initialized
   */
  isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Reset the registry (useful for testing)
   */
  reset(): void {
    this.initialized = false;
    this.skillMatcher = null;
    this.skillExecutor = null;
    this.mcpClient = null;
  }
}

/**
 * Convenience function to get the skill registry instance
 */
export function getSkillRegistry(): SkillRegistry {
  return SkillRegistry.getInstance();
}
