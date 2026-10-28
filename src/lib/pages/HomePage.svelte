<script lang="ts">
  import { app } from '$lib/state/app.svelte';
  import DeviceTree from '$lib/components/DeviceTree.svelte';
  import Preview from '$lib/components/Preview.svelte';
  import ColorPanel from '$lib/components/ColorPanel.svelte';
  import Effects from '$lib/components/Effects.svelte';
  import KbdPanel from '$lib/components/KbdPanel.svelte';
  import KbdPreview from '$lib/components/KbdPreview.svelte';
</script>

<div class="home">
  <div class="tree"><DeviceTree /></div>
  {#if app.selected === 'kbd' && app.kbd}
    <div class="view"><KbdPreview /></div>
    <div class="side"><KbdPanel /></div>
    <div class="fx"><Effects /></div>
  {:else}
    <div class="view"><Preview /></div>
    <div class="side"><ColorPanel /></div>
    <div class="fx"><Effects /></div>
  {/if}
</div>

<style>
  .home {
    flex: 1;
    display: grid;
    grid-template-columns: 250px minmax(0, 1fr) 330px;
    grid-template-rows: minmax(0, 1.25fr) minmax(0, 1fr);
    grid-template-areas: 'tree view side' 'tree fx side';
    gap: 14px;
    min-height: 0;
  }
  .home > div { display: grid; min-height: 0; min-width: 0; }
  .tree { grid-area: tree; }
  .view { grid-area: view; }
  .side { grid-area: side; }
  .fx { grid-area: fx; }
</style>
