<script lang="ts">
    let {
        width,
        height,
    }: {
        width?: string;
        height?: string;
    } = $props();
</script>

<img src="/favicon.png" alt="VRCTV Logo" {width} {height} class="mx-auto overflow-visible" />
