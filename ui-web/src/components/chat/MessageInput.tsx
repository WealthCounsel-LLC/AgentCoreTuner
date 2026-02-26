import { createSignal } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';

interface ExtendedMessage {
  role: string;
  content: string;
  thinking?: string;
  payload?: string;
}

const MessageInput: Component = () => {
  const [input, setInput] = createSignal('');

  const canSend = () => {
    return (
      input().trim() &&
      !store.loading().sending &&
      store.authStatus() === 'valid' &&
      store.agents().length > 0
    );
  };

  const buildPayload = () => {
    const agent = store.agents()[store.selectedAgentIdx()];
    const model = store.models()[store.selectedModelIdx()];
    const memoryIdx = store.selectedMemoryIdx();
    const memory = memoryIdx > 0 ? store.memories()[memoryIdx - 1] : null;

    return {
      profile: store.selectedProfile(),
      region: store.selectedRegion(),
      agent_arn: agent?.arn || '',
      session_id: store.sessionId() || `session-${Date.now()}`,
      qualifier: store.selectedQualifier(),
      runtime_user_id: store.runtimeUserId(),
      input: input(),
      model_id: model?.id || '',
      system_prompt: store.systemPrompt(),
      memory_id: memory?.id || '',
      extra_payload: {},
    };
  };

  const sendMessage = async () => {
    if (!canSend()) return;

    const messageContent = input().trim();
    const payload = buildPayload();

    // Add user message to chat
    const userMessage: ExtendedMessage = {
      role: 'user',
      content: messageContent,
      payload: JSON.stringify(payload, null, 2),
    };
    store.setMessages([...store.messages(), userMessage as any]);

    // Clear input
    setInput('');

    // Update loading state
    const loading = store.loading();
    store.loading; // trigger reactivity
    Object.assign(loading, { sending: true });

    try {
      // Store the session ID if we generated a new one
      if (!store.sessionId() && payload.session_id) {
        store.setSessionId(payload.session_id);
      }

      // Invoke agent
      const [thinking, response, debugLog] = await tauri.invokeAgent(payload);

      // Add assistant response
      const assistantMessage: ExtendedMessage = {
        role: 'assistant',
        content: response,
        thinking: thinking || undefined,
      };
      store.setMessages([...store.messages(), assistantMessage as any]);

      // Log debug output if dev mode
      if (store.devMode() && debugLog) {
        console.log('[Debug Log]', debugLog);
      }
    } catch (err) {
      store.showStatus(`Failed to send message: ${err}`, true);

      // Add error message
      const errorMessage: ExtendedMessage = {
        role: 'assistant',
        content: `Error: ${err}`,
      };
      store.setMessages([...store.messages(), errorMessage as any]);
    } finally {
      Object.assign(loading, { sending: false });
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  return (
    <div class="border-t border-border p-4 bg-surface">
      <div class="flex gap-3">
        <textarea
          value={input()}
          onInput={(e) => setInput(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
          disabled={store.loading().sending}
          rows={3}
          class="flex-1 bg-surface-2 border border-border rounded-standard px-4 py-3 text-sm
                 resize-none focus:outline-none focus:ring-2 focus:ring-accent
                 disabled:opacity-50 placeholder:text-text-dim/50"
        />
        <button
          onClick={sendMessage}
          disabled={!canSend()}
          class="px-6 py-2 bg-accent text-crust rounded-standard font-medium
                 hover:bg-accent-hover transition-colors disabled:opacity-50
                 flex items-center gap-2 self-end"
        >
          <span class="material-icons text-sm">send</span>
          Send
        </button>
      </div>
    </div>
  );
};

export default MessageInput;
