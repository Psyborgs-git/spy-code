/**
 * Skills Validation Tests
 * Validates that skill files exist and have required sections
 */

import * as fs from 'fs';
import * as path from 'path';

function validateSkillFile(filePath: string): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  if (!fs.existsSync(filePath)) {
    errors.push(`File does not exist: ${filePath}`);
    return { valid: false, errors };
  }
  
  const content = fs.readFileSync(filePath, 'utf8');
  
  // Check for required sections
  const requiredSections = ['## When to Use', '## Available Tools', '## Best Practices'];
  
  for (const section of requiredSections) {
    if (!content.includes(section)) {
      errors.push(`Missing required section: ${section}`);
    }
  }
  
  return { valid: errors.length === 0, errors };
}

function validateAllSkills() {
  const skillsDir = path.join(__dirname, '../../skills');
  const universalDir = path.join(skillsDir, 'universal');
  const environmentsDir = path.join(skillsDir, 'environments');
  
  const universalSkills = ['code-navigation.md', 'call-graph-analysis.md', 'semantic-search.md', 'change-tracking.md', 'graph-visualization.md'];
  const environmentSkills = ['cursor-skills.md', 'windsurf-skills.md', 'copilot-skills.md', 'claude-skills.md'];
  
  let allValid = true;
  const allErrors: string[] = [];
  
  console.log('Validating universal skills...');
  for (const skill of universalSkills) {
    const skillPath = path.join(universalDir, skill);
    const result = validateSkillFile(skillPath);
    
    if (result.valid) {
      console.log(`✓ ${skill}`);
    } else {
      console.log(`✗ ${skill}`);
      allErrors.push(...result.errors.map(e => `${skill}: ${e}`));
      allValid = false;
    }
  }
  
  console.log('\nValidating environment-specific skills...');
  for (const skill of environmentSkills) {
    const skillPath = path.join(environmentsDir, skill);
    const result = validateSkillFile(skillPath);
    
    if (result.valid) {
      console.log(`✓ ${skill}`);
    } else {
      console.log(`✗ ${skill}`);
      allErrors.push(...result.errors.map(e => `${skill}: ${e}`));
      allValid = false;
    }
  }
  
  if (allValid) {
    console.log('\n✓ All skills are valid');
  } else {
    console.log('\n✗ Some skills have errors:');
    allErrors.forEach(e => console.log(`  - ${e}`));
  }
  
  return allValid;
}

// Run validation if executed directly
if (require.main === module) {
  const valid = validateAllSkills();
  process.exit(valid ? 0 : 1);
}

export { validateSkillFile, validateAllSkills };
