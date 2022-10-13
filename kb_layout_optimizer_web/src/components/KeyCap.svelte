<script lang="ts">
  export let selected: boolean | undefined;
  export let position: { x: number; y: number };
  export let onClick: (e: MouseEvent) => void;

  const PX_PER_UNIT = 54;
</script>

<div
  class={`keycap${selected ? ' selected' : ''}`}
  style="left: {position.x * PX_PER_UNIT}px; top: {position.y * PX_PER_UNIT}px"
  on:click={onClick}
>
  <div class="keycap-top"><slot /></div>
</div>

<style lang="scss">
  .keycap {
    position: absolute;
    width: 54px;
    height: 54px;
    border-width: 1px;
    border-radius: 5px;
    border-color: black;
    border-style: solid;
    background-color: #cccccc;

    &:hover {
      border-color: green;
    }

    &.selected {
      border-color: red;
    }
  }

  .keycap-top {
    position: absolute;
    top: 3px;
    left: 5px;
    width: 42px;
    height: 42px;
    border-style: solid;
    border-width: 1px;
    border-color: rgba(0, 0, 0, 0.1);
    background-color: #fcfcfc;
    border-radius: 3px;
  }
</style>
