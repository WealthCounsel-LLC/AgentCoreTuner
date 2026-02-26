import type { Component } from 'solid-js';
import ChatConfig from './ChatConfig';
import MessageList from './MessageList';
import MessageInput from './MessageInput';
import ChatRatings from './ChatRatings';

const ChatTab: Component = () => {
  return (
    <div class="h-full flex flex-col gap-4">
      {/* Top: Chat config (full width) */}
      <div class="flex-shrink-0">
        <ChatConfig />
      </div>

      {/* Bottom: Messages (full width, takes remaining space) */}
      <div class="flex-1 flex flex-col bg-surface rounded-standard border border-border overflow-hidden min-h-0">
        <MessageList />
        <ChatRatings />
        <MessageInput />
      </div>
    </div>
  );
};

export default ChatTab;
