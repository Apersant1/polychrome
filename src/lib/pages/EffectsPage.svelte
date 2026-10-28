<script lang="ts">
  import ColorPanel from '$lib/components/ColorPanel.svelte';
  import Effects from '$lib/components/Effects.svelte';
</script>

<div class="split">
  <Effects all />
  <ColorPanel />
</div>

<style>
  .split { flex: 1; display: grid; grid-template-columns: minmax(0, 1fr) 350px; gap: 14px; min-height: 0; align-items: start; }
</style>
