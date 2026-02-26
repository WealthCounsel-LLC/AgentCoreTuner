import { Show, For } from 'solid-js';
import type { Component } from 'solid-js';
import type { AgentRuntimeDetail } from '../../types';

interface RuntimeDetailViewProps {
  isOpen: boolean;
  runtime: AgentRuntimeDetail | null;
  onClose: () => void;
}

const RuntimeDetailView: Component<RuntimeDetailViewProps> = (props) => {
  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleString();
    } catch {
      return dateStr;
    }
  };

  const formatTimeout = (seconds: number) => {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
  };

  const DetailRow: Component<{ label: string; value: string; mono?: boolean }> = (rowProps) => (
    <div class="grid grid-cols-3 gap-2 py-2 border-b border-border last:border-0">
      <span class="text-text-dim text-sm">{rowProps.label}</span>
      <span class={`col-span-2 text-sm ${rowProps.mono ? 'font-mono' : ''} break-all`}>
        {rowProps.value || '-'}
      </span>
    </div>
  );

  return (
    <Show when={props.isOpen && props.runtime}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-2xl max-h-[90vh] flex flex-col">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <div>
              <h2 class="text-lg font-semibold">{props.runtime!.name}</h2>
              <p class="text-xs text-text-dim font-mono">{props.runtime!.id}</p>
            </div>
            <button
              onClick={props.onClose}
              class="p-1 hover:bg-surface-2 rounded transition-colors"
            >
              <span class="material-icons">close</span>
            </button>
          </div>

          <div class="flex-1 overflow-auto p-4">
            {/* Basic Info */}
            <div class="mb-6">
              <h3 class="text-sm font-semibold text-text-dim mb-2 uppercase tracking-wide">
                Basic Information
              </h3>
              <div class="bg-surface-2 rounded-standard p-3">
                <DetailRow label="ARN" value={props.runtime!.arn} mono />
                <DetailRow label="Version" value={props.runtime!.version} />
                <DetailRow label="Status" value={props.runtime!.status} />
                <DetailRow label="Description" value={props.runtime!.description} />
              </div>
            </div>

            {/* Code Configuration */}
            <div class="mb-6">
              <h3 class="text-sm font-semibold text-text-dim mb-2 uppercase tracking-wide">
                Code Configuration
              </h3>
              <div class="bg-surface-2 rounded-standard p-3">
                <DetailRow label="Runtime" value={props.runtime!.code_runtime} />
                <DetailRow label="Entry Point" value={props.runtime!.code_entry_point} mono />
                <DetailRow label="S3 Bucket" value={props.runtime!.code_s3_bucket} mono />
                <DetailRow label="S3 Prefix" value={props.runtime!.code_s3_prefix} mono />
              </div>
            </div>

            {/* Network & Timeouts */}
            <div class="mb-6">
              <h3 class="text-sm font-semibold text-text-dim mb-2 uppercase tracking-wide">
                Network & Timeouts
              </h3>
              <div class="bg-surface-2 rounded-standard p-3">
                <DetailRow label="Protocol" value={props.runtime!.protocol} />
                <DetailRow label="Network Mode" value={props.runtime!.network_mode} />
                <DetailRow
                  label="Idle Timeout"
                  value={`${formatTimeout(props.runtime!.idle_session_timeout)} (${props.runtime!.idle_session_timeout}s)`}
                />
                <DetailRow
                  label="Max Lifetime"
                  value={`${formatTimeout(props.runtime!.max_lifetime)} (${props.runtime!.max_lifetime}s)`}
                />
              </div>
            </div>

            {/* IAM */}
            <div class="mb-6">
              <h3 class="text-sm font-semibold text-text-dim mb-2 uppercase tracking-wide">
                IAM Role
              </h3>
              <div class="bg-surface-2 rounded-standard p-3">
                <DetailRow label="Role ARN" value={props.runtime!.role_arn} mono />
              </div>
            </div>

            {/* Environment Variables */}
            <Show when={Object.keys(props.runtime!.environment_variables || {}).length > 0}>
              <div class="mb-6">
                <h3 class="text-sm font-semibold text-text-dim mb-2 uppercase tracking-wide">
                  Environment Variables
                </h3>
                <div class="bg-surface-2 rounded-standard p-3">
                  <For each={Object.entries(props.runtime!.environment_variables || {})}>
                    {([key, value]) => (
                      <DetailRow label={key} value={value} mono />
                    )}
                  </For>
                </div>
              </div>
            </Show>

            {/* Timestamps */}
            <div>
              <h3 class="text-sm font-semibold text-text-dim mb-2 uppercase tracking-wide">
                Timestamps
              </h3>
              <div class="bg-surface-2 rounded-standard p-3">
                <DetailRow label="Created" value={formatDate(props.runtime!.created_at)} />
                <DetailRow label="Last Updated" value={formatDate(props.runtime!.last_updated_at)} />
              </div>
            </div>
          </div>

          <div class="flex justify-end p-4 border-t border-border">
            <button
              onClick={props.onClose}
              class="px-4 py-2 text-sm rounded-standard hover:bg-surface-2 transition-colors"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default RuntimeDetailView;
