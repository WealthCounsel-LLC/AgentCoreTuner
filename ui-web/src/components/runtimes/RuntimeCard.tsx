import type { Component } from 'solid-js';
import type { Agent } from '../../types';

interface RuntimeCardProps {
  agent: Agent;
  isSelected: boolean;
  onClick: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onViewDetails: () => void;
}

const RuntimeCard: Component<RuntimeCardProps> = (props) => {
  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString();
    } catch {
      return dateStr;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active':
      case 'running':
        return 'bg-success/20 text-success';
      case 'pending':
      case 'updating':
        return 'bg-warning/20 text-warning';
      case 'failed':
      case 'error':
        return 'bg-error/20 text-error';
      default:
        return 'bg-surface-2 text-text-dim';
    }
  };

  return (
    <div
      class={`p-4 rounded-standard border cursor-pointer transition-colors ${
        props.isSelected
          ? 'bg-accent/10 border-accent'
          : 'bg-surface border-border hover:bg-surface-2'
      }`}
      onClick={props.onClick}
    >
      <div class="flex items-start justify-between">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <h3 class="font-medium text-text truncate">{props.agent.name}</h3>
            <span class={`px-2 py-0.5 rounded-full text-xs ${getStatusColor(props.agent.status)}`}>
              {props.agent.status}
            </span>
          </div>
          <p class="text-xs text-text-dim mt-1 font-mono truncate">
            {props.agent.id}
          </p>
        </div>
        <div class="flex items-center gap-1 ml-4">
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onViewDetails();
            }}
            class="p-1 hover:bg-surface-2 rounded transition-colors"
            title="View Details"
          >
            <span class="material-icons text-sm text-text-dim">visibility</span>
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onEdit();
            }}
            class="p-1 hover:bg-surface-2 rounded transition-colors"
            title="Edit"
          >
            <span class="material-icons text-sm text-text-dim">edit</span>
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onDelete();
            }}
            class="p-1 hover:bg-error/20 rounded transition-colors"
            title="Delete"
          >
            <span class="material-icons text-sm text-error">delete</span>
          </button>
        </div>
      </div>

      <div class="mt-3 flex items-center gap-4 text-xs text-text-dim">
        <div class="flex items-center gap-1">
          <span class="material-icons text-xs">label</span>
          <span>v{props.agent.version}</span>
        </div>
        <div class="flex items-center gap-1">
          <span class="material-icons text-xs">schedule</span>
          <span>Updated {formatDate(props.agent.last_updated)}</span>
        </div>
      </div>
    </div>
  );
};

export default RuntimeCard;
