/**
 * Skill Matcher
 * Matches agent requests to appropriate skills based on intent and context
 */
import { SkillDefinition } from './skill-loader';
export interface MatchResult {
    skill: SkillDefinition;
    confidence: number;
    reason: string;
}
export declare class SkillMatcher {
    private skills;
    constructor(skills: Map<string, SkillDefinition>);
    /**
     * Match a user request to the most appropriate skill
     */
    match(request: string, context?: any): MatchResult[];
    /**
     * Calculate confidence score for a skill match
     */
    private calculateConfidence;
    /**
     * Get a human-readable reason for the match
     */
    private getMatchReason;
    /**
     * Get the best matching skill
     */
    getBestMatch(request: string, context?: any): MatchResult | null;
    /**
     * Get skills that use a specific tool
     */
    getSkillsUsingTool(tool: string): SkillDefinition[];
    /**
     * Get all skills
     */
    private getAllSkills;
}
