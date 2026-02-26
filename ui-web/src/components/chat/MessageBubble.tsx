import { Show, createSignal } from 'solid-js';
import type { Component } from 'solid-js';

interface MessageBubbleProps {
  role: 'user' | 'assistant';
  content: string;
  thinking?: string;
  payload?: string;
}

const MessageBubble: Component<MessageBubbleProps> = (props) => {
  const [showThinking, setShowThinking] = createSignal(false);
  const [showPayload, setShowPayload] = createSignal(false);

  const isUser = () => props.role === 'user';

  const copyContent = async () => {
    try {
      await navigator.clipboard.writeText(props.content);
    } catch {
      // Clipboard API may not be available
    }
  };

  return (
    <div class={`flex ${isUser() ? 'justify-end' : 'justify-start'} mb-4`}>
      <div
        class={`max-w-[80%] rounded-standard p-4 ${
          isUser()
            ? 'bg-user-bubble text-text'
            : 'bg-bot-bubble border border-border text-text'
        }`}
      >
        {/* Message content */}
        <div class="whitespace-pre-wrap break-words text-sm">{props.content}</div>

        {/* Thinking section (assistant only) */}
        <Show when={!isUser() && props.thinking}>
          <div class="mt-3 border-t border-border/50 pt-3">
            <button
              onClick={() => setShowThinking(!showThinking())}
              class="flex items-center gap-1 text-xs text-text-dim hover:text-text transition-colors"
            >
              <span class="material-icons text-sm">
                {showThinking() ? 'expand_less' : 'expand_more'}
              </span>
              Thinking
            </button>
            <Show when={showThinking()}>
              <pre class="mt-2 text-xs text-text-dim bg-crust p-2 rounded overflow-auto max-h-40">
                {props.thinking}
              </pre>
            </Show>
          </div>
        </Show>

        {/* Payload preview (user only) */}
        <Show when={isUser() && props.payload}>
          <div class="mt-3 border-t border-border/50 pt-3">
            <button
              onClick={() => setShowPayload(!showPayload())}
              class="flex items-center gap-1 text-xs text-text-dim hover:text-text transition-colors"
            >
              <span class="material-icons text-sm">
                {showPayload() ? 'expand_less' : 'expand_more'}
              </span>
              Payload
            </button>
            <Show when={showPayload()}>
              <pre class="mt-2 text-xs text-text-dim bg-crust p-2 rounded overflow-auto max-h-40">
                {props.payload}
              </pre>
            </Show>
          </div>
        </Show>

        {/* Actions */}
        <div class="mt-2 flex justify-end">
          <button
            onClick={copyContent}
            title="Copy message"
            class="p-1 rounded hover:bg-black/10 transition-colors"
          >
            <span class="material-icons text-text-dim text-sm">content_copy</span>
          </button>
        </div>
      </div>
    </div>
  );
};

export default MessageBubble;
