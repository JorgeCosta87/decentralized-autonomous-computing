/**
 * TypeScript types for DAC configuration structures
 * Based on docs/configs.yaml schema
 */

export interface NetworkConfig {
  name: string;
  description: string;
  version: string;
  allowed_models: AllowedModel[];
}

export interface AllowedModel {
  name: string;
  provider: string;
  parameters: number;
  use_case: string;
  vram_requirements: {
    fp16: number;
    int8: number;
    int4: number;
  };
  context_length: number;
  cost_per_1m_tokens: number;
}

export interface NodeConfig {
  node_id: string;
  name: string;
  description: string;
  location?: string;
  hardware: {
    cpu: {
      model: string;
      cores: number;
      threads: number;
      base_clock_ghz: number;
      boost_clock_ghz: number;
      architecture: string;
    };
    gpu: Array<{
      name: string;
      vram_gb: number;
    }>;
  };
  software: {
    cuda_version: string;
  };
  available_models: AvailableModel[];
}

export interface AvailableModel {
  model_name: string;
  quantization: 'fp16' | 'int8' | 'int4';
  vram_required_gb: number;
  gpu_index: number;
  max_concurrent_requests: number;
}

export interface ToolArg {
  name: string;
  description: string;
  type: 'string' | 'number' | 'boolean';
  required: boolean;
  default?: string | number | boolean;
}

export interface ToolConfig {
  name: string;
  description: string;
  args: ToolArg[];
  parameters: {
    memory_access?: boolean;
    network_access?: boolean;
    rate_limit?: number;
  };
}

export interface ToolsConfig {
  tools: ToolConfig[];
}

export interface AgentConfig {
  name: string;
  description: string;
  version: string;
  author: string;
  model: {
    model_name: string;
    context: {
      specialization: string; // Can be inline or IPFS CID
    };
  };
  capabilities: {
    tools: string[]; // Tool IDs from tools_config
  };
  memory: {
    type: 'episodic' | 'semantic' | 'hybrid';
    max_memory_entries: number;
    memory_retention_days: number;
    compression: boolean;
  };
}

export interface GoalResource {
  name: string;
  description: string;
  type: 'website' | 'document' | 'image' | 'code' | 'data';
  format?: string; // For IPFS resources (markdown, svg, pdf, etc.)
  url?: string; // For web resources
  cid?: string; // For IPFS resources (use either url or cid, not both)
}

export interface GoalDeliverable {
  name: string;
  description?: string;
  type: 'code' | 'test_suite' | 'document' | 'diagram' | 'data' | 'markdown';
  format: string; // Specific format/framework (markdown, python_fastapi, pytest, mermaid, etc.)
}

export interface GoalSpecification {
  title: string;
  description: string; // Can be inline or IPFS CID
  category: string;
  resources?: GoalResource[];
  deliverables: GoalDeliverable[];
  success_criteria: string[];
  validation: {
    type: 'llm_judge' | 'voting' | 'automated' | 'manual';
  };
}

export interface ConfigSchema {
  file_fields: {
    network_config: {
      inline_only: boolean;
    };
    node_config: {
      inline_only: boolean;
    };
    tools_config: {
      inline_only: boolean;
    };
    agent_config: {
      model: {
        specialization: {
          type: string;
          can_be_file: boolean;
          file_types: string[];
          description: string;
        };
      };
    };
    goal_specification: {
      description: {
        type: string;
        can_be_file: boolean;
        file_types: string[];
        description: string;
      };
      resources: {
        type: string;
        can_be_file: boolean;
      };
      deliverables: {
        type: string;
        can_be_file: boolean;
      };
      success_criteria: {
        type: string;
        can_be_file: boolean;
      };
      validation: {
        type: string;
        can_be_file: boolean;
      };
    };
  };
  best_practices: string[];
  supported_file_types: {
    [key: string]: {
      extensions: string[];
      mime_types: string[];
      description: string;
    };
  };
}
