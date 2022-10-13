<script lang="ts">
  import KeyCap from './components/KeyCap.svelte';

  const PX_PER_UNIT = 54;

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
  let selectedButtonIndices: number[] = [];

  const handleKeyDown = (e: KeyboardEvent) => {
    // console.log(e);
    switch (e.key) {
      case 'ArrowUp':
        selectedButtonIndices.forEach((idx) => {
          buttons[idx].position.y -= 0.1;
        });
        break;
      case 'ArrowRight':
        selectedButtonIndices.forEach((idx) => {
          buttons[idx].position.x += 0.1;
        });
        break;
      case 'ArrowDown':
        selectedButtonIndices.forEach((idx) => {
          buttons[idx].position.y += 0.1;
        });
        break;
      case 'ArrowLeft':
        selectedButtonIndices.forEach((idx) => {
          buttons[idx].position.x -= 0.1;
        });
        break;
      case 'Delete':
        selectedButtonIndices.forEach((idx) => {
          buttons[idx].position.x -= 0.1;
        });
        break;
      case 'q':
        console.log(selectedButtonIndices);
        buttons = buttons.filter((_, i) => !selectedButtonIndices.includes(i));
        selectedButtonIndices = [];
        break;
      case 'e':
        buttons = [
          ...buttons,
          {
            finger: Finger.THUMB,
            hand: Hand.LEFT,
            matrix_position: { x: 0, y: 0 },
            position: { x: 0, y: 0 },
          },
        ];
        break;
      case 'w':
        buttons = [
          ...buttons,
          ...selectedButtonIndices
            .map((i) => buttons[i])
            .map((button) => {
              return {
                finger: Finger.THUMB,
                hand: Hand.LEFT,
                matrix_position: { x: 0, y: 0 },
                position: {
                  x: button.position.x,
                  y: button.position.y - 1,
                },
              };
            }),
        ];
        selectedButtonIndices = selectedButtonIndices.map(
          (_, i) => buttons.length - 1 - i,
        );
        break;
      case 'a':
        buttons = [
          ...buttons,
          ...selectedButtonIndices
            .map((i) => buttons[i])
            .map((button) => {
              return {
                finger: Finger.THUMB,
                hand: Hand.LEFT,
                matrix_position: { x: 0, y: 0 },
                position: {
                  x: button.position.x - 1,
                  y: button.position.y,
                },
              };
            }),
        ];
        selectedButtonIndices = selectedButtonIndices.map(
          (_, i) => buttons.length - 1 - i,
        );
        break;
      case 's':
        buttons = [
          ...buttons,
          ...selectedButtonIndices
            .map((i) => buttons[i])
            .map((button) => {
              return {
                finger: Finger.THUMB,
                hand: Hand.LEFT,
                matrix_position: { x: 0, y: 0 },
                position: {
                  x: button.position.x,
                  y: button.position.y + 1,
                },
              };
            }),
        ];
        selectedButtonIndices = selectedButtonIndices.map(
          (_, i) => buttons.length - 1 - i,
        );
        break;
      case 'd':
        buttons = [
          ...buttons,
          ...selectedButtonIndices
            .map((i) => buttons[i])
            .map((button) => {
              return {
                finger: Finger.THUMB,
                hand: Hand.LEFT,
                matrix_position: { x: 0, y: 0 },
                position: {
                  x: button.position.x + 1,
                  y: button.position.y,
                },
              };
            }),
        ];
        selectedButtonIndices = selectedButtonIndices.map(
          (_, i) => buttons.length - 1 - i,
        );
        break;
    }
  };

  let dragStart: { x: number; y: number } | null = null;
  let dragEnd: { x: number; y: number } | null = null;
  const handleMouseDown = (e: MouseEvent) => {
    if (e.shiftKey) {
      dragStart = { x: e.pageX, y: e.pageY };
      dragEnd = { x: e.pageX, y: e.pageY };
    }
  };

  const handleMouseUp = (_e: MouseEvent) => {
    dragStart = null;
    dragEnd = null;
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (dragStart !== null) {
      dragEnd = { x: e.pageX, y: e.pageY };
      const dragRect = {
        left: Math.min(dragStart.x, dragEnd.x),
        top: Math.min(dragStart.y, dragEnd.y),
        width: Math.abs(dragStart.x - dragEnd.x),
        height: Math.abs(dragStart.y - dragEnd.y),
      };
      selectedButtonIndices = buttons
        .map((button, index) => {
          return { button, index };
        })
        .filter(({ button }) => {
          const buttonRect = {
            left: button.position.x * PX_PER_UNIT,
            top: button.position.y * PX_PER_UNIT,
            width: PX_PER_UNIT,
            height: PX_PER_UNIT,
          };
          return !(
            buttonRect.top + buttonRect.height < dragRect.top ||
            buttonRect.top > dragRect.top + dragRect.height ||
            buttonRect.left + buttonRect.width < dragRect.left ||
            buttonRect.left > dragRect.left + dragRect.width
          );
        })
        .map(({ index }) => index);
    }
  };
</script>

<svelte:window
  on:keydown={handleKeyDown}
  on:mousedown={handleMouseDown}
  on:mouseup={handleMouseUp}
  on:mousemove={handleMouseMove}
/>

{#if dragStart !== null && dragEnd !== null}
  <div
    style="position: absolute; left: {Math.min(
      dragStart.x,
      dragEnd.x,
    )}px; top: {Math.min(dragStart.y, dragEnd.y)}px; width: {Math.abs(
      dragStart.x - dragEnd.x,
    )}px; height: {Math.abs(
      dragStart.y - dragEnd.y,
    )}px; border: 1px solid rgba(130, 163, 255 , 0.9); background: rgba(130, 163, 255 , 0.4); z-index: 999"
  />
{/if}

<div style="position: relative; user-select: none">
  {#each buttons as button, idx}
    <KeyCap
      position={button.position}
      selected={selectedButtonIndices.includes(idx)}
      onClick={(e) => {
        e.stopPropagation();
        if (e.ctrlKey) {
          if (selectedButtonIndices.includes(idx)) {
            selectedButtonIndices = selectedButtonIndices.filter(
              (i) => i !== idx,
            );
          } else {
            selectedButtonIndices = [...selectedButtonIndices, idx];
          }
        } else {
          if (
            selectedButtonIndices.length === 1 &&
            selectedButtonIndices.includes(idx)
          ) {
            selectedButtonIndices = [];
          } else {
            selectedButtonIndices = [idx];
          }
        }
      }}
      >{button.position.x.toFixed(1)}
      {button.position.y.toFixed(1)}</KeyCap
    >
  {/each}
</div>

<div style="top: 500px; position: absolute">
  <div>
    Start:
    {#if dragStart !== null}
      {dragStart.x} {dragStart.y}
    {/if}
  </div>
  <div>
    End:
    {#if dragEnd !== null}
      {dragEnd.x} {dragEnd.y}
    {/if}
  </div>
</div>
