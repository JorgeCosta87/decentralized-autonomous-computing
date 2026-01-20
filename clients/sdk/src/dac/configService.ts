import yaml from 'js-yaml';
import type {
  NetworkConfig,
  NodeConfig,
  ToolsConfig,
  AgentConfig,
  GoalSpecification,
  ConfigSchema,
} from './configTypes.js';

/**
 * Service for loading and providing DAC configuration schemas and information
 */
export class ConfigService {
  private static schemaCache: ConfigSchema | null = null;

  /**
   * Load config schema from YAML content
   */
  static loadSchemaFromYaml(yamlContent: string): ConfigSchema {
    const parsed = yaml.load(yamlContent) as any;
    return parsed.schema as ConfigSchema;
  }

  /**
   * Get default config schema (embedded in SDK)
   * This provides information about field types and file capabilities
   */
  static getDefaultSchema(): ConfigSchema {
    if (this.schemaCache) {
      return this.schemaCache;
    }

    // Default schema based on docs/configs.yaml
    this.schemaCache = {
      file_fields: {
        network_config: {
          inline_only: true,
        },
        node_config: {
          inline_only: true,
        },
        tools_config: {
          inline_only: true,
        },
        agent_config: {
          model: {
            specialization: {
              type: 'string',
              can_be_file: true,
              file_types: ['text', 'markdown'],
              description:
                'Specialization - defines the agent\'s expertise and capabilities, can be inline YAML string or IPFS CID to text/markdown file',
            },
          },
        },
        goal_specification: {
          description: {
            type: 'string',
            can_be_file: true,
            file_types: ['text', 'markdown'],
            description:
              'Goal description - can be inline YAML string or IPFS CID to text/markdown file',
          },
          resources: {
            type: 'array',
            can_be_file: false,
          },
          deliverables: {
            type: 'array',
            can_be_file: false,
          },
          success_criteria: {
            type: 'array',
            can_be_file: false,
          },
          validation: {
            type: 'object',
            can_be_file: false,
          },
        },
      },
      best_practices: [
        'Use inline strings for short text (< 500 chars)',
        'Use IPFS CIDs for longer content or reusable templates',
        'Keep YAML files readable - prefer inline for examples',
        'Store large files (code, documentation) on IPFS and reference by CID',
        'Use consistent indentation (2 spaces recommended)',
        'Quote strings with special characters or colons',
        'Use YAML anchors (&) and aliases (*) for repeated values',
        'Validate YAML syntax before uploading to IPFS',
      ],
      supported_file_types: {
        text: {
          extensions: ['.txt', '.md', '.markdown'],
          mime_types: ['text/plain', 'text/markdown'],
          description: 'Plain text and markdown files',
        },
        code: {
          extensions: ['.py', '.js', '.ts', '.rs', '.go', '.java', '.cpp', '.c'],
          mime_types: [
            'text/x-python',
            'application/javascript',
            'text/x-rust',
            'text/x-go',
          ],
          description: 'Source code files',
        },
        json: {
          extensions: ['.json'],
          mime_types: ['application/json'],
          description: 'JSON data files',
        },
        yaml: {
          extensions: ['.yaml', '.yml'],
          mime_types: ['application/x-yaml', 'text/yaml'],
          description: 'YAML configuration files',
        },
      },
    };

    return this.schemaCache;
  }

  /**
   * Parse network config from YAML
   */
  static parseNetworkConfig(yamlContent: string): NetworkConfig {
    const parsed = yaml.load(yamlContent) as any;
    return parsed.network_config as NetworkConfig;
  }

  /**
   * Parse node config from YAML
   */
  static parseNodeConfig(yamlContent: string): NodeConfig {
    const parsed = yaml.load(yamlContent) as any;
    return parsed.node_config as NodeConfig;
  }

  /**
   * Parse tools config from YAML
   */
  static parseToolsConfig(yamlContent: string): ToolsConfig {
    const parsed = yaml.load(yamlContent) as any;
    return parsed.tools_config as ToolsConfig;
  }

  /**
   * Parse agent config from YAML
   */
  static parseAgentConfig(yamlContent: string): AgentConfig {
    const parsed = yaml.load(yamlContent) as any;
    return parsed.agent_config as AgentConfig;
  }

  /**
   * Parse goal specification from YAML
   */
  static parseGoalSpecification(yamlContent: string): GoalSpecification {
    const parsed = yaml.load(yamlContent) as any;
    return parsed.goal_specification as GoalSpecification;
  }

  /**
   * Check if a field can reference a file/IPFS CID
   */
  static canFieldBeFile(
    configType: 'network_config' | 'node_config' | 'tools_config' | 'agent_config' | 'goal_specification',
    fieldPath: string
  ): boolean {
    const schema = this.getDefaultSchema();
    const configSchema = schema.file_fields[configType];

    if (configType === 'network_config' || configType === 'node_config' || configType === 'tools_config') {
      return !(configSchema as any).inline_only;
    }

    if (configType === 'agent_config') {
      const agentSchema = configSchema as any;
      if (fieldPath === 'model.specialization') {
        return agentSchema.model.specialization.can_be_file;
      }
    }

    if (configType === 'goal_specification') {
      const goalSchema = configSchema as any;
      if (fieldPath === 'description') {
        return goalSchema.description.can_be_file;
      }
    }

    return false;
  }

  /**
   * Get allowed file types for a field
   */
  static getAllowedFileTypes(
    configType: 'agent_config' | 'goal_specification',
    fieldPath: string
  ): string[] {
    const schema = this.getDefaultSchema();
    const configSchema = schema.file_fields[configType];

    if (configType === 'agent_config') {
      const agentSchema = configSchema as any;
      if (fieldPath === 'model.specialization') {
        return agentSchema.model.specialization.file_types;
      }
    }

    if (configType === 'goal_specification') {
      const goalSchema = configSchema as any;
      if (fieldPath === 'description') {
        return goalSchema.description.file_types;
      }
    }

    return [];
  }
}
