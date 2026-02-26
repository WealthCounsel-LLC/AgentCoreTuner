import { For, Show, createEffect } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import MessageBubble from './MessageBubble';

interface ExtendedMessage {
  role: string;
  content: string;
  thinking?: string;
  payload?: string;
}

const MessageList: Component = () => {
  let containerRef: HTMLDivElement | undefined;

  // Auto-scroll to bottom when messages change
  createEffect(() => {
    const messages = store.messages();
    if (messages.length > 0 && containerRef) {
      containerRef.scrollTop = containerRef.scrollHeight;
    }
  });

  return (
    <div
      ref={containerRef}
      class="flex-1 overflow-auto p-4"
    >
      <Show
        when={store.messages().length > 0}
        fallback={
          <div class="h-full flex items-center justify-center">
            <div class="text-center text-text-dim">
              <span class="material-icons text-4xl mb-2">chat_bubble_outline</span>
              <p>No messages yet</p>
              <p class="text-sm mt-1">Send a message to start the conversation</p>
            </div>
          </div>
        }
      >
        <For each={store.messages() as ExtendedMessage[]}>
          {(message) => (
            <MessageBubble
              role={message.role as 'user' | 'assistant'}
              content={message.content}
              thinking={message.thinking}
              payload={message.payload}
            />
          )}
        </For>
      </Show>

      {/* Loading indicator */}
      <Show when={store.loading().sending}>
        <div class="flex justify-start mb-4">
          <div class="bg-bot-bubble border border-border rounded-standard p-4">
            <div class="flex items-center gap-2 text-text-dim">
              <span class="material-icons animate-spin text-sm">sync</span>
              <span class="text-sm">Thinking...</span>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default MessageList;
