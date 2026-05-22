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

export class SkillMatcher {
  private skills: Map<string, SkillDefinition>;

  constructor(skills: Map<string, SkillDefinition>) {
    this.skills = skills;
  }

  /**
   * Match a user request to the most appropriate skill
   */
  match(request: string, context?: any): MatchResult[] {
    const results: MatchResult[] = [];
    const lowerRequest = request.toLowerCase();

    for (const skill of this.skills.values()) {
      const confidence = this.calculateConfidence(request, lowerRequest, skill, context);
      
      if (confidence > 0.3) {
        results.push({
          skill,
          confidence,
          reason: this.getMatchReason(request, lowerRequest, skill, confidence)
        });
      }
    }

    // Sort by confidence descending
    results.sort((a, b) => b.confidence - a.confidence);

    return results;
  }

  /**
   * Calculate confidence score for a skill match
   */
  private calculateConfidence(
    request: string,
    lowerRequest: string,
    skill: SkillDefinition,
    context?: any
  ): number {
    let score = 0;

    // Check if request matches "when to use" criteria
    for (const criterion of skill.whenToUse) {
      const lowerCriterion = criterion.toLowerCase();
      if (lowerRequest.includes(lowerCriterion)) {
        score += 0.4;
      }
    }

    // Check for keywords in skill name
    const skillNameLower = skill.name.toLowerCase();
    if (lowerRequest.includes(skillNameLower)) {
      score += 0.3;
    }

    // Check for tool mentions
    for (const tool of skill.availableTools) {
      if (lowerRequest.includes(tool.toLowerCase())) {
        score += 0.2;
      }
    }

    // Check for example query similarity
    for (const example of skill.exampleQueries) {
      const lowerExample = example.toLowerCase();
      const words = lowerExample.split(/\s+/);
      const matches = words.filter(word => word.length > 3 && lowerRequest.includes(word));
      if (matches.length > 0) {
        score += 0.1 * (matches.length / words.length);
      }
    }

    // Context-based adjustments
    if (context) {
      if (context.operation === 'search' && skill.availableTools.includes('search')) {
        score += 0.2;
      }
      if (context.operation === 'navigation' && skill.name.toLowerCase().includes('navigation')) {
        score += 0.2;
      }
    }

    return Math.min(score, 1.0);
  }

  /**
   * Get a human-readable reason for the match
   */
  private getMatchReason(
    request: string,
    lowerRequest: string,
    skill: SkillDefinition,
    confidence: number
  ): string {
    const reasons: string[] = [];

    for (const criterion of skill.whenToUse) {
      if (lowerRequest.includes(criterion.toLowerCase())) {
        reasons.push(`matches criterion "${criterion}"`);
      }
    }

    if (lowerRequest.includes(skill.name.toLowerCase())) {
      reasons.push(`mentions "${skill.name}"`);
    }

    if (reasons.length === 0) {
      return `general match (confidence: ${confidence.toFixed(2)})`;
    }

    return reasons.join(', ');
  }

  /**
   * Get the best matching skill
   */
  getBestMatch(request: string, context?: any): MatchResult | null {
    const matches = this.match(request, context);
    return matches.length > 0 ? matches[0] : null;
  }

  /**
   * Get skills that use a specific tool
   */
  getSkillsUsingTool(tool: string): SkillDefinition[] {
    return this.getAllSkills().filter(skill =>
      skill.availableTools.some(t => t.toLowerCase().includes(tool.toLowerCase()))
    );
  }

  /**
   * Get all skills
   */
  private getAllSkills(): SkillDefinition[] {
    return Array.from(this.skills.values());
  }
}
