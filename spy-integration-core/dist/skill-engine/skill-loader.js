"use strict";
/**
 * Skill Loader
 * Loads and validates skill definitions from the skills directory
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.SkillLoader = void 0;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
class SkillLoader {
    constructor(skillsDir = 'skills') {
        this.skills = new Map();
        this.skillsDir = skillsDir;
    }
    /**
     * Load all skills from the skills directory
     */
    async loadAll() {
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
    async loadSkillsFromDirectory(dir) {
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
    async parseSkillFile(filePath) {
        try {
            const content = fs.readFileSync(filePath, 'utf8');
            const lines = content.split('\n');
            const skill = {
                id: path.basename(filePath, '.md'),
                name: '',
                description: '',
                whenToUse: [],
                availableTools: [],
                exampleQueries: [],
                bestPractices: [],
                commonPatterns: []
            };
            let currentSection = '';
            let currentPattern = null;
            for (const line of lines) {
                // Parse headers
                if (line.startsWith('# ')) {
                    skill.name = line.substring(2).trim();
                }
                else if (line.startsWith('## ')) {
                    currentSection = line.substring(3).trim().toLowerCase().replace(/\s+/g, '-');
                }
                else if (line.startsWith('### ')) {
                    if (currentSection === 'common-patterns') {
                        currentPattern = { name: line.substring(4).trim(), description: '', steps: [] };
                    }
                }
                // Parse content based on section
                if (currentSection === 'when-to-use' && line.startsWith('- ')) {
                    skill.whenToUse.push(line.substring(2).trim());
                }
                else if (currentSection === 'available-tools' && line.startsWith('- ')) {
                    skill.availableTools.push(line.substring(2).trim());
                }
                else if (currentSection === 'example-queries' && line.startsWith('```')) {
                    // Skip code block markers
                }
                else if (currentSection === 'example-queries' && line.trim() && !line.startsWith('```')) {
                    skill.exampleQueries.push(line.trim());
                }
                else if (currentSection === 'best-practices' && line.match(/^\d+\./)) {
                    skill.bestPractices.push(line.replace(/^\d+\.\s*/, '').trim());
                }
                else if (currentSection === 'common-patterns' && currentPattern) {
                    if (line.startsWith('- ')) {
                        currentPattern.steps.push(line.substring(2).trim());
                    }
                    else if (line.trim() && !line.startsWith('###')) {
                        currentPattern.description += line + ' ';
                    }
                }
            }
            // Add completed pattern
            if (currentPattern && currentPattern.steps.length > 0) {
                skill.commonPatterns.push(currentPattern);
            }
            // Validate skill
            if (!skill.name || skill.whenToUse.length === 0) {
                console.warn(`[SkillLoader] Invalid skill file: ${filePath}`);
                return null;
            }
            return skill;
        }
        catch (error) {
            console.error(`[SkillLoader] Error parsing skill file ${filePath}:`, error);
            return null;
        }
    }
    /**
     * Get a skill by ID
     */
    getSkill(id) {
        return this.skills.get(id);
    }
    /**
     * Get all skills
     */
    getAllSkills() {
        return Array.from(this.skills.values());
    }
    /**
     * Get skills by category (universal or environment-specific)
     */
    getSkillsByCategory(category) {
        return this.getAllSkills().filter(skill => {
            if (category === 'universal') {
                return !skill.id.includes('-skills');
            }
            return skill.id.includes('-skills');
        });
    }
}
exports.SkillLoader = SkillLoader;
