import type { Component } from 'solid-js';
import AgentSelector from './AgentSelector';
import ModelSelector from './ModelSelector';
import MemorySelector from './MemorySelector';
import PromptSelector from './PromptSelector';
import RecipeSelector from './RecipeSelector';
import QualifierSelector from './QualifierSelector';
import UserIdInput from './UserIdInput';
import SessionIdInput from './SessionIdInput';

const ChatConfig: Component = () => {
  return (
    <div class="bg-surface rounded-standard border border-border p-4">
      {/* Row 1: Agent, Model, Memory, Prompt */}
      <div class="grid grid-cols-4 gap-4 mb-3">
        <AgentSelector />
        <ModelSelector />
        <MemorySelector />
        <PromptSelector />
      </div>

      {/* Row 2: Recipe, Qualifier, User ID, Session ID */}
      <div class="grid grid-cols-4 gap-4">
        <RecipeSelector />
        <QualifierSelector />
        <UserIdInput />
        <SessionIdInput />
      </div>
    </div>
  );
};

export default ChatConfig;
