import { createSignal } from 'solid-js';
import type { Component } from 'solid-js';
import RatingWidget from './RatingWidget';

const ChatRatings: Component = () => {
  const [personalityRating, setPersonalityRating] = createSignal(0);
  const [accuracyRating, setAccuracyRating] = createSignal(0);

  return (
    <div class="flex items-center gap-6 px-4 py-2 bg-surface border-t border-border">
      <RatingWidget
        label="Personality"
        value={personalityRating()}
        onChange={setPersonalityRating}
      />
      <RatingWidget
        label="Accuracy"
        value={accuracyRating()}
        onChange={setAccuracyRating}
      />
    </div>
  );
};

export default ChatRatings;
