"use strict";
/**
 * Skill Registry
 * Central registry for managing available skills
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.SkillRegistry = void 0;
exports.getSkillRegistry = getSkillRegistry;
const skill_loader_1 = require("./skill-loader");
const skill_matcher_1 = require("./skill-matcher");
const skill_executor_1 = require("./skill-executor");
class SkillRegistry {
    constructor() {
        this.skillMatcher = null;
        this.skillExecutor = null;
        this.mcpClient = null;
        this.initialized = false;
        this.skillLoader = new skill_loader_1.SkillLoader();
    }
    /**
     * Get singleton instance
     */
    static getInstance() {
        if (!SkillRegistry.instance) {
            SkillRegistry.instance = new SkillRegistry();
        }
        return SkillRegistry.instance;
    }
    /**
     * Initialize the skill registry
     */
    async initialize(mcpClient, skillsDir) {
        if (this.initialized) {
            return;
        }
        this.mcpClient = mcpClient;
        if (skillsDir) {
            this.skillLoader = new skill_loader_1.SkillLoader(skillsDir);
        }
        // Load all skills
        await this.skillLoader.loadAll();
        // Initialize matcher and executor
        const skills = this.skillLoader.getAllSkills();
        const skillsMap = new Map(skills.map(s => [s.id, s]));
        this.skillMatcher = new skill_matcher_1.SkillMatcher(skillsMap);
        this.skillExecutor = new skill_executor_1.SkillExecutor(mcpClient, skillsMap);
        this.initialized = true;
    }
    /**
     * Match a request to skills
     */
    match(request, context) {
        if (!this.skillMatcher) {
            throw new Error('Skill registry not initialized');
        }
        return this.skillMatcher.match(request, context);
    }
    /**
     * Get the best matching skill
     */
    getBestMatch(request, context) {
        if (!this.skillMatcher) {
            throw new Error('Skill registry not initialized');
        }
        return this.skillMatcher.getBestMatch(request, context);
    }
    /**
     * Execute a skill
     */
    async execute(skillId, context) {
        if (!this.skillExecutor) {
            throw new Error('Skill registry not initialized');
        }
        return this.skillExecutor.execute(skillId, context);
    }
    /**
     * Execute a skill pattern
     */
    async executePattern(skillId, patternName) {
        if (!this.skillExecutor) {
            throw new Error('Skill registry not initialized');
        }
        return this.skillExecutor.executePattern(skillId, patternName);
    }
    /**
     * Get a skill by ID
     */
    getSkill(id) {
        return this.skillLoader.getSkill(id);
    }
    /**
     * Get all skills
     */
    getAllSkills() {
        return this.skillLoader.getAllSkills();
    }
    /**
     * Get skills by category
     */
    getSkillsByCategory(category) {
        return this.skillLoader.getSkillsByCategory(category);
    }
    /**
     * Get skills using a specific tool
     */
    getSkillsUsingTool(tool) {
        if (!this.skillMatcher) {
            throw new Error('Skill registry not initialized');
        }
        return this.skillMatcher.getSkillsUsingTool(tool);
    }
    /**
     * Check if initialized
     */
    isInitialized() {
        return this.initialized;
    }
    /**
     * Reset the registry (useful for testing)
     */
    reset() {
        this.initialized = false;
        this.skillMatcher = null;
        this.skillExecutor = null;
        this.mcpClient = null;
    }
}
exports.SkillRegistry = SkillRegistry;
/**
 * Convenience function to get the skill registry instance
 */
function getSkillRegistry() {
    return SkillRegistry.getInstance();
}
