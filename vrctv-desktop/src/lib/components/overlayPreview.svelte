<script lang="ts">
  import type { OverlayItem } from "../../bindings";

  const {
    overlay,
    class: className = "",
  }: {
    overlay: OverlayItem;
    class?: string;
  } = $props();
</script>

{#if overlay.isIframe}
  <iframe src={overlay.url} title="Preview {overlay.name}" class={className}>
  </iframe>
{:else}
  <img src={overlay.url} alt="Preview {overlay.name}" class={className} />
{/if}
