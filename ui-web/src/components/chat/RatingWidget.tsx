import { For, createSignal } from 'solid-js';
import type { Component } from 'solid-js';

interface RatingWidgetProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
}

const RatingWidget: Component<RatingWidgetProps> = (props) => {
  const [hovered, setHovered] = createSignal(0);

  return (
    <div class="flex items-center gap-2">
      <span class="text-xs text-text-dim uppercase tracking-wide">{props.label}</span>
      <div class="flex">
        <For each={[1, 2, 3, 4, 5]}>
          {(star) => (
            <button
              onMouseEnter={() => setHovered(star)}
              onMouseLeave={() => setHovered(0)}
              onClick={() => props.onChange(props.value === star ? 0 : star)}
              class="p-0.5 transition-colors"
            >
              <span
                class={`material-icons text-lg ${
                  (hovered() || props.value) >= star
                    ? 'text-yellow'
                    : 'text-overlay-0'
                }`}
              >
                {(hovered() || props.value) >= star ? 'star' : 'star_outline'}
              </span>
            </button>
          )}
        </For>
      </div>
    </div>
  );
};

export default RatingWidget;
