/**
 * Skill Loader
 * Loads and validates skill definitions from the skills directory
 */

import * as fs from 'fs';
import * as path from 'path';

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

export class SkillLoader {
  private skillsDir: string;
  private skills: Map<string, SkillDefinition> = new Map();

  constructor(skillsDir: string = 'skills') {
    this.skillsDir = skillsDir;
  }

  /**
   * Load all skills from the skills directory
   */
  async loadAll(): Promise<Map<string, SkillDefinition>> {
    const universalDir = path.join(this.skillsDir, 'universal');
    const environmentsDir = path.join(this.skillsDir, 'environments');

    // Load universal skills
    await this.loadSkillsFromDirectory(universalDir);

    // Load environment-specific skills
    await this.loadSkillsFromDirectory(environmentsDir);

    return this.skills;
  }

  /**
   * Load skills from a specific directory
   */
  private async loadSkillsFromDirectory(dir: string): Promise<void> {
    if (!fs.existsSync(dir)) {
      console.warn(`[SkillLoader] Skills directory not found: ${dir}`);
      return;
    }

    const files = fs.readdirSync(dir);
    
    for (const file of files) {
      if (file.endsWith('.md')) {
        const filePath = path.join(dir, file);
        const skill = await this.parseSkillFile(filePath);
        if (skill) {
          this.skills.set(skill.id, skill);
        }
      }
    }
  }

  /**
   * Parse a skill markdown file into a SkillDefinition
   */
  private async parseSkillFile(filePath: string): Promise<SkillDefinition | null> {
    try {
      const content = fs.readFileSync(filePath, 'utf8');
      const lines = content.split('\n');

      const skill: Partial<SkillDefinition> = {
        id: path.basename(filePath, '.md'),
        name: '',
        description: '',
        whenToUse: [],
        availableTools: [],
        exampleQueries: [],
        bestPractices: [],
        commonPatterns: []
      };

      let currentSection: string = '';
      let currentPattern: any = null;

      for (const line of lines) {
        // Parse headers
        if (line.startsWith('# ')) {
          skill.name = line.substring(2).trim();
        } else if (line.startsWith('## ')) {
          currentSection = line.substring(3).trim().toLowerCase().replace(/\s+/g, '-');
        } else if (line.startsWith('### ')) {
          if (currentSection === 'common-patterns') {
            currentPattern = { name: line.substring(4).trim(), description: '', steps: [] };
          }
        }

        // Parse content based on section
        if (currentSection === 'when-to-use' && line.startsWith('- ')) {
          skill.whenToUse!.push(line.substring(2).trim());
        } else if (currentSection === 'available-tools' && line.startsWith('- ')) {
          skill.availableTools!.push(line.substring(2).trim());
        } else if (currentSection === 'example-queries' && line.startsWith('```')) {
          // Skip code block markers
        } else if (currentSection === 'example-queries' && line.trim() && !line.startsWith('```')) {
          skill.exampleQueries!.push(line.trim());
        } else if (currentSection === 'best-practices' && line.match(/^\d+\./)) {
          skill.bestPractices!.push(line.replace(/^\d+\.\s*/, '').trim());
        } else if (currentSection === 'common-patterns' && currentPattern) {
          if (line.startsWith('- ')) {
            currentPattern.steps.push(line.substring(2).trim());
          } else if (line.trim() && !line.startsWith('###')) {
            currentPattern.description += line + ' ';
          }
        }
      }

      // Add completed pattern
      if (currentPattern && currentPattern.steps.length > 0) {
        skill.commonPatterns!.push(currentPattern);
      }

      // Validate skill
      if (!skill.name || skill.whenToUse!.length === 0) {
        console.warn(`[SkillLoader] Invalid skill file: ${filePath}`);
        return null;
      }

      return skill as SkillDefinition;
    } catch (error) {
      console.error(`[SkillLoader] Error parsing skill file ${filePath}:`, error);
      return null;
    }
  }

  /**
   * Get a skill by ID
   */
  getSkill(id: string): SkillDefinition | undefined {
    return this.skills.get(id);
  }

  /**
   * Get all skills
   */
  getAllSkills(): SkillDefinition[] {
    return Array.from(this.skills.values());
  }

  /**
   * Get skills by category (universal or environment-specific)
   */
  getSkillsByCategory(category: 'universal' | 'environments'): SkillDefinition[] {
    return this.getAllSkills().filter(skill => {
      if (category === 'universal') {
        return !skill.id.includes('-skills');
      }
      return skill.id.includes('-skills');
    });
  }
}
