import fs from 'fs-extra';
import path from 'path';
import { fileURLToPath } from 'url';

// When using ES modules with Node.js, import.meta.url provides the URL of the current module
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Templates directory
const TEMPLATES_DIR = path.join(__dirname, '../templates');

/**
 * Processes a template file with the provided variables
 * @param templateName Template file name
 * @param variables Variables to replace in the template
 * @returns Processed template content
 */
export async function processTemplate(templateName: string, variables: Record<string, string>): Promise<string> {
  const templatePath = path.join(TEMPLATES_DIR, templateName);
  
  try {
    // Read the template file
    const templateContent = await fs.readFile(templatePath, 'utf-8');
    
    // Replace variables in the template
    let processedContent = templateContent;
    
    for (const [key, value] of Object.entries(variables)) {
      const regex = new RegExp(`{{${key}}}`, 'g');
      processedContent = processedContent.replace(regex, value);
    }
    
    return processedContent;
  } catch (error) {
    throw new Error(`Failed to process template ${templateName}: ${error instanceof Error ? error.message : String(error)}`);
  }
}

/**
 * Writes a processed template to a file
 * @param templateName Template file name
 * @param outputPath Output file path
 * @param variables Variables to replace in the template
 */
export async function writeTemplateToFile(templateName: string, outputPath: string, variables: Record<string, string>): Promise<void> {
  try {
    const processedContent = await processTemplate(templateName, variables);
    await fs.writeFile(outputPath, processedContent);
  } catch (error) {
    throw new Error(`Failed to write template to file ${outputPath}: ${error instanceof Error ? error.message : String(error)}`);
  }
}

/**
 * Generates a Dockerfile for an MCP server
 * @param outputPath Output file path
 * @param templateName Template file name
 * @param variables Variables to replace in the template
 */
export async function generateDockerfile(outputPath: string, templateName: string, variables: Record<string, string>): Promise<void> {
  await writeTemplateToFile(templateName, outputPath, variables);
}

/**
 * Lists available template files
 * @returns List of available template files
 */
export async function listTemplates(): Promise<string[]> {
  try {
    const files = await fs.readdir(TEMPLATES_DIR);
    return files.filter(file => file.endsWith('.template'));
  } catch (error) {
    throw new Error(`Failed to list templates: ${error instanceof Error ? error.message : String(error)}`);
  }
}