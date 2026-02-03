<script lang="ts">
    import icon from "../../favicon.png";

    const {
        width,
        height,
    }: {
        width?: string;
        height?: string;
    } = $props();
</script>

<img src={icon} {width} {height} alt="VRCTV Logo" />