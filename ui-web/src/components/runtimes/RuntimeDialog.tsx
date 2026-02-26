import { createSignal, createEffect, For, Show } from 'solid-js';
import type { Component } from 'solid-js';
import type { AgentRuntimeDetail, CreateAgentRuntimeParams, RecipeEntry } from '../../types';
import { store } from '../../stores/app';

interface RuntimeDialogProps {
  isOpen: boolean;
  mode: 'create' | 'edit';
  runtime: AgentRuntimeDetail | null;
  initialRecipe?: RecipeEntry | null;
  onClose: () => void;
  onSave: (params: CreateAgentRuntimeParams) => void;
}

const CODE_RUNTIMES = [
  'PYTHON_3_10',
  'PYTHON_3_11',
  'PYTHON_3_12',
  'PYTHON_3_13',
];

const PROTOCOLS = ['HTTP', 'MCP', 'A2A'];

const RuntimeDialog: Component<RuntimeDialogProps> = (props) => {
  const [name, setName] = createSignal('');
  const [description, setDescription] = createSignal('');
  const [roleArn, setRoleArn] = createSignal('');
  const [s3Bucket, setS3Bucket] = createSignal('');
  const [s3Prefix, setS3Prefix] = createSignal('');
  const [codeRuntime, setCodeRuntime] = createSignal('PYTHON_3_13');
  const [entryPoint, setEntryPoint] = createSignal('main.py');
  const [protocol, setProtocol] = createSignal('HTTP');
  const [idleTimeout, setIdleTimeout] = createSignal(900);
  const [maxLifetime, setMaxLifetime] = createSignal(28800);
  const [networkMode, setNetworkMode] = createSignal('TENANT');
  const [envVars, setEnvVars] = createSignal<Record<string, string>>({
    DEFAULT_MODEL_ID: '',
    DEFAULT_SYSTEM_PROMPT: '',
  });

  // Reset form when dialog opens
  createEffect(() => {
    if (props.isOpen) {
      if (props.mode === 'edit' && props.runtime) {
        // Edit mode: populate from existing runtime
        setName(props.runtime.name);
        setDescription(props.runtime.description);
        setRoleArn(props.runtime.role_arn);
        setS3Bucket(props.runtime.code_s3_bucket);
        setS3Prefix(props.runtime.code_s3_prefix);
        setCodeRuntime(props.runtime.code_runtime);
        setEntryPoint(props.runtime.code_entry_point);
        setProtocol(props.runtime.protocol);
        setIdleTimeout(props.runtime.idle_session_timeout);
        setMaxLifetime(props.runtime.max_lifetime);
        setNetworkMode(props.runtime.network_mode);
        setEnvVars(props.runtime.environment_variables);
      } else if (props.initialRecipe) {
        // Create from snapshot: populate from recipe
        const recipe = props.initialRecipe;

        // Get prompt content if recipe references a prompt
        let systemPrompt = recipe.system_prompt || '';
        if (recipe.prompt_id && !systemPrompt) {
          const prompt = store.prompts().find(p => p.id === recipe.prompt_id);
          if (prompt) {
            systemPrompt = prompt.content;
          }
        }

        setName(recipe.name.toLowerCase().replace(/[^a-z0-9-]/g, '-'));
        setDescription(recipe.description);
        setRoleArn(''); // User must provide
        setS3Bucket('');
        setS3Prefix('');
        setCodeRuntime('PYTHON_3_13');
        setEntryPoint('main.py');
        setProtocol('HTTP');
        setIdleTimeout(900);
        setMaxLifetime(28800);
        setNetworkMode('TENANT');
        setEnvVars({
          DEFAULT_MODEL_ID: recipe.model_id || '',
          DEFAULT_SYSTEM_PROMPT: systemPrompt,
          DEFAULT_MEMORY_ID: recipe.memory_id || '',
          RECIPE_ID: recipe.id,
          RECIPE_NAME: recipe.name,
        });
      } else {
        // Blank create mode
        setName('');
        setDescription('');
        setRoleArn('');
        setS3Bucket('');
        setS3Prefix('');
        setCodeRuntime('PYTHON_3_13');
        setEntryPoint('main.py');
        setProtocol('HTTP');
        setIdleTimeout(900);
        setMaxLifetime(28800);
        setNetworkMode('TENANT');
        setEnvVars({
          DEFAULT_MODEL_ID: '',
          DEFAULT_SYSTEM_PROMPT: '',
        });
      }
    }
  });

  const handleSubmit = (e: Event) => {
    e.preventDefault();

    const params: CreateAgentRuntimeParams = {
      profile: store.selectedProfile(),
      region: store.selectedRegion(),
      name: name(),
      agent_runtime_id: props.runtime?.id || '',
      description: description(),
      role_arn: roleArn(),
      network_mode: networkMode(),
      protocol: protocol(),
      idle_session_timeout: idleTimeout(),
      max_lifetime: maxLifetime(),
      code_s3_bucket: s3Bucket(),
      code_s3_prefix: s3Prefix(),
      code_runtime: codeRuntime(),
      code_entry_point: entryPoint(),
      environment_variables: envVars(),
    };

    props.onSave(params);
  };

  const canSubmit = () => name().trim().length > 0 && roleArn().trim().length > 0;

  const updateEnvVar = (key: string, value: string) => {
    setEnvVars(prev => ({ ...prev, [key]: value }));
  };

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-2xl max-h-[90vh] flex flex-col">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <div>
              <h2 class="text-lg font-semibold">
                {props.mode === 'create'
                  ? (props.initialRecipe ? 'Build Runtime from Snapshot' : 'Create Runtime')
                  : 'Edit Runtime'}
              </h2>
              <Show when={props.initialRecipe}>
                <p class="text-sm text-accent">
                  Based on: {props.initialRecipe!.name}
                </p>
              </Show>
            </div>
            <button
              onClick={props.onClose}
              class="p-1 hover:bg-surface-2 rounded transition-colors"
            >
              <span class="material-icons">close</span>
            </button>
          </div>

          <form onSubmit={handleSubmit} class="flex-1 overflow-auto p-4">
            <div class="grid gap-4">
              {/* Name */}
              <div>
                <label class="block text-sm font-medium mb-1">
                  Name <span class="text-error">*</span>
                </label>
                <input
                  type="text"
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                  placeholder="my-agent-runtime"
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         focus:outline-none focus:ring-2 focus:ring-accent"
                />
              </div>

              {/* Description */}
              <div>
                <label class="block text-sm font-medium mb-1">Description</label>
                <textarea
                  value={description()}
                  onInput={(e) => setDescription(e.currentTarget.value)}
                  placeholder="Describe what this runtime does..."
                  rows={2}
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         resize-none focus:outline-none focus:ring-2 focus:ring-accent"
                />
              </div>

              {/* Role ARN */}
              <div>
                <label class="block text-sm font-medium mb-1">
                  IAM Role ARN <span class="text-error">*</span>
                </label>
                <input
                  type="text"
                  value={roleArn()}
                  onInput={(e) => setRoleArn(e.currentTarget.value)}
                  placeholder="arn:aws:iam::123456789:role/AgentCoreRole"
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         font-mono focus:outline-none focus:ring-2 focus:ring-accent"
                />
              </div>

              {/* S3 Configuration */}
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium mb-1">S3 Bucket</label>
                  <input
                    type="text"
                    value={s3Bucket()}
                    onInput={(e) => setS3Bucket(e.currentTarget.value)}
                    placeholder="my-agent-bucket"
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-accent"
                  />
                </div>
                <div>
                  <label class="block text-sm font-medium mb-1">S3 Prefix</label>
                  <input
                    type="text"
                    value={s3Prefix()}
                    onInput={(e) => setS3Prefix(e.currentTarget.value)}
                    placeholder="agents/my-agent"
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-accent"
                  />
                </div>
              </div>

              {/* Code Runtime and Entry Point */}
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium mb-1">Code Runtime</label>
                  <select
                    value={codeRuntime()}
                    onChange={(e) => setCodeRuntime(e.currentTarget.value)}
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-accent"
                  >
                    <For each={CODE_RUNTIMES}>
                      {(runtime) => <option value={runtime}>{runtime}</option>}
                    </For>
                  </select>
                </div>
                <div>
                  <label class="block text-sm font-medium mb-1">Entry Point</label>
                  <input
                    type="text"
                    value={entryPoint()}
                    onInput={(e) => setEntryPoint(e.currentTarget.value)}
                    placeholder="main.py"
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           font-mono focus:outline-none focus:ring-2 focus:ring-accent"
                  />
                </div>
              </div>

              {/* Protocol and Network Mode */}
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium mb-1">Protocol</label>
                  <select
                    value={protocol()}
                    onChange={(e) => setProtocol(e.currentTarget.value)}
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-accent"
                  >
                    <For each={PROTOCOLS}>
                      {(p) => <option value={p}>{p}</option>}
                    </For>
                  </select>
                </div>
                <div>
                  <label class="block text-sm font-medium mb-1">Network Mode</label>
                  <select
                    value={networkMode()}
                    onChange={(e) => setNetworkMode(e.currentTarget.value)}
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-accent"
                  >
                    <option value="TENANT">TENANT</option>
                    <option value="PUBLIC">PUBLIC</option>
                  </select>
                </div>
              </div>

              {/* Timeouts */}
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium mb-1">
                    Idle Timeout (seconds)
                  </label>
                  <input
                    type="number"
                    value={idleTimeout()}
                    onInput={(e) => setIdleTimeout(parseInt(e.currentTarget.value) || 900)}
                    min={60}
                    max={28800}
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-accent"
                  />
                  <p class="text-xs text-text-dim mt-1">60 - 28800 (default: 900)</p>
                </div>
                <div>
                  <label class="block text-sm font-medium mb-1">
                    Max Lifetime (seconds)
                  </label>
                  <input
                    type="number"
                    value={maxLifetime()}
                    onInput={(e) => setMaxLifetime(parseInt(e.currentTarget.value) || 28800)}
                    min={60}
                    max={28800}
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-accent"
                  />
                  <p class="text-xs text-text-dim mt-1">60 - 28800 (default: 28800)</p>
                </div>
              </div>

              {/* Environment Variables */}
              <div>
                <label class="block text-sm font-medium mb-2">
                  Environment Variables
                  <Show when={props.initialRecipe}>
                    <span class="text-xs text-accent ml-2">(populated from snapshot)</span>
                  </Show>
                </label>
                <div class="space-y-2 bg-surface-2 rounded-standard p-3">
                  <div class="grid grid-cols-3 gap-2 items-center">
                    <span class="text-xs text-text-dim">DEFAULT_MODEL_ID</span>
                    <input
                      type="text"
                      value={envVars()['DEFAULT_MODEL_ID'] || ''}
                      onInput={(e) => updateEnvVar('DEFAULT_MODEL_ID', e.currentTarget.value)}
                      placeholder="claude-3-sonnet"
                      class="col-span-2 bg-surface border border-border rounded px-2 py-1 text-sm
                             font-mono focus:outline-none focus:ring-1 focus:ring-accent"
                    />
                  </div>
                  <div class="grid grid-cols-3 gap-2 items-start">
                    <span class="text-xs text-text-dim pt-1">DEFAULT_SYSTEM_PROMPT</span>
                    <textarea
                      value={envVars()['DEFAULT_SYSTEM_PROMPT'] || ''}
                      onInput={(e) => updateEnvVar('DEFAULT_SYSTEM_PROMPT', e.currentTarget.value)}
                      placeholder="You are a helpful assistant."
                      rows={3}
                      class="col-span-2 bg-surface border border-border rounded px-2 py-1 text-sm
                             font-mono focus:outline-none focus:ring-1 focus:ring-accent resize-none"
                    />
                  </div>
                  <div class="grid grid-cols-3 gap-2 items-center">
                    <span class="text-xs text-text-dim">DEFAULT_MEMORY_ID</span>
                    <input
                      type="text"
                      value={envVars()['DEFAULT_MEMORY_ID'] || ''}
                      onInput={(e) => updateEnvVar('DEFAULT_MEMORY_ID', e.currentTarget.value)}
                      placeholder="memory-id"
                      class="col-span-2 bg-surface border border-border rounded px-2 py-1 text-sm
                             font-mono focus:outline-none focus:ring-1 focus:ring-accent"
                    />
                  </div>
                  <Show when={envVars()['RECIPE_ID']}>
                    <div class="grid grid-cols-3 gap-2 items-center text-text-dim">
                      <span class="text-xs">RECIPE_ID</span>
                      <span class="col-span-2 text-xs font-mono truncate">
                        {envVars()['RECIPE_ID']}
                      </span>
                    </div>
                  </Show>
                  <Show when={envVars()['RECIPE_NAME']}>
                    <div class="grid grid-cols-3 gap-2 items-center text-text-dim">
                      <span class="text-xs">RECIPE_NAME</span>
                      <span class="col-span-2 text-xs font-mono truncate">
                        {envVars()['RECIPE_NAME']}
                      </span>
                    </div>
                  </Show>
                </div>
              </div>
            </div>
          </form>

          <div class="flex justify-end gap-3 p-4 border-t border-border">
            <button
              type="button"
              onClick={props.onClose}
              class="px-4 py-2 text-sm rounded-standard hover:bg-surface-2 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              onClick={handleSubmit}
              disabled={!canSubmit()}
              class="px-4 py-2 text-sm bg-accent text-crust rounded-standard font-medium
                     hover:bg-accent-hover transition-colors disabled:opacity-50"
            >
              {props.mode === 'create' ? 'Create' : 'Save'}
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default RuntimeDialog;
