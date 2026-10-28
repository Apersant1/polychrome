<script lang="ts">
  import { app } from '$lib/state/app.svelte';
  import DeviceTree from '$lib/components/DeviceTree.svelte';
  import Preview from '$lib/components/Preview.svelte';
  import ColorPanel from '$lib/components/ColorPanel.svelte';
  import Effects from '$lib/components/Effects.svelte';
  import ApplyPanel from '$lib/components/ApplyPanel.svelte';
  import KbdPanel from '$lib/components/KbdPanel.svelte';
  import KbdPreview from '$lib/components/KbdPreview.svelte';
  import Profiles from '$lib/components/Profiles.svelte';
</script>

<div class="home">
  <div class="tree"><DeviceTree /></div>
  {#if app.selected === 'kbd' && app.kbd}
    <div class="view"><KbdPreview /></div>
    <div class="kbdpanel"><KbdPanel /></div>
    <div class="fx"><Effects /></div>
  {:else}
    <div class="view"><Preview /></div>
    <div class="color"><ColorPanel /></div>
    <div class="fx"><Effects /></div>
    <div class="targets"><ApplyPanel /></div>
  {/if}
  <div class="prof"><Profiles /></div>
</div>

<style>
  .home {
    flex: 1;
    display: grid;
    grid-template-columns: 250px minmax(0, 1fr) 330px;
    grid-template-rows: minmax(0, 1.25fr) minmax(0, 1fr) auto;
    grid-template-areas: 'tree view color' 'tree fx targets' 'prof prof prof';
    gap: 14px;
    min-height: 0;
  }
  .home > div { display: grid; min-height: 0; min-width: 0; }
  .tree { grid-area: tree; }
  .view { grid-area: view; }
  .color { grid-area: color; }
  .fx { grid-area: fx; }
  .targets { grid-area: targets; }
  .prof { grid-area: prof; }
  .kbdpanel { grid-column: 3; grid-row: 1 / 3; }
</style>
