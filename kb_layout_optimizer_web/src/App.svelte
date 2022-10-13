<script lang="ts">
  import KeyCap from './components/KeyCap.svelte';

  const COLORS = [
    'cornflowerblue',
    'coral',
    'aquamarine',
    'darkkhaki',
    'darkseagreen',
    'hotpink',
    'lavenderblush',
    'palegreen',
    'peru',
    'pink',
    'skyblue',
    'wheat',
  ];

  interface Button {
    hand: Hand;
    finger: Finger;
    position: { x: number; y: number };
    matrix_position: { x: number; y: number };
  }

  enum Hand {
    LEFT = 'Left',
    RIGHT = 'Right',
  }

  enum Finger {
    THUMB = 'Thumb',
    POINTER = 'Pointer',
    MIDDLE = 'Middle',
    RING = 'Ring',
    PINKY = 'Pinky',
  }

  let buttons: Button[] = [];
  let selectedButtonIndex: number | null = null;

  $: console.log(selectedButtonIndex);
</script>

<div>
  <h2>Settings</h2>
  <button
    on:click={() => {
      buttons = [
        ...buttons,
        {
          finger: Finger.THUMB,
          hand: Hand.LEFT,
          matrix_position: { x: 0, y: 0 },
          position: { x: 0, y: 0 },
        },
      ];
    }}
  >
    Add button
  </button>
</div>
<div style="position: relative">
  {#each buttons as button, index}
    <KeyCap
      position={button.position}
      selected={index === selectedButtonIndex}
      onClick={(e) => {
        e.stopPropagation();
        if (index === selectedButtonIndex) {
          selectedButtonIndex = null;
        } else {
          selectedButtonIndex = index;
        }
      }}
    >
      A
    </KeyCap>
  {/each}
</div>
