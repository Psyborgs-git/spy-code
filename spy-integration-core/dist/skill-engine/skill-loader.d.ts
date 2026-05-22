/**
 * Skill Loader
 * Loads and validates skill definitions from the skills directory
 */
export interface SkillDefinition {
    id: string;
    name: string;
    description: string;
    whenToUse: string[];
    availableTools: string[];
    exampleQueries: string[];
    bestPractices: string[];
    commonPatterns: Array<{
        name: string;
        description: string;
        steps: string[];
    }>;
}
export declare class SkillLoader {
    private skillsDir;
    private skills;
    constructor(skillsDir?: string);
    /**
     * Load all skills from the skills directory
     */
    loadAll(): Promise<Map<string, SkillDefinition>>;
    /**
     * Load skills from a specific directory
     */
    private loadSkillsFromDirectory;
    /**
     * Parse a skill markdown file into a SkillDefinition
     */
    private parseSkillFile;
    /**
     * Get a skill by ID
     */
    getSkill(id: string): SkillDefinition | undefined;
    /**
     * Get all skills
     */
    getAllSkills(): SkillDefinition[];
    /**
     * Get skills by category (universal or environment-specific)
     */
    getSkillsByCategory(category: 'universal' | 'environments'): SkillDefinition[];
}
