<script lang="ts">
    import { fade } from "svelte/transition";
    import { cubicIn, cubicOut } from "svelte/easing";
    import { onMount } from "svelte";

    const {
        cogSpinning = true,
        width,
        height,
    }: {
        cogSpinning?: boolean;
        width?: string;
        height?: string;
    } = $props();
    let cogVisible = $state(false);
    let heartVisible = $state(true);
    let eyesVisible = $state(false);

    onMount(() => {
        setTimeout(() => {
            cogVisible = true;
            eyesVisible = true;
        }, 900);

        setTimeout(() => {
            heartVisible = false;
        }, 2900);
    });
</script>

<svg
    id="logo"
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 88.854 94.007"
    {width}
    {height}
    class="overflow-visible"
>
    {#if cogVisible}
        <path
            d="M44.418 0c-1.828.007-3.6.145-5.738.394l-2.47 7.41c-7.52 1.925-10.297 4.403-10.694 4.403-2.358-1.09-4.64-2.342-6.98-3.468-4.115 3.282-5.737 5.071-8.358 8.357 1.162 2.325 2.334 4.644 3.468 6.982 0 .33-2.537 3.4-4.404 10.693l-7.41 2.47c-.128 1.102-.44 8.045 0 11.82l7.41 2.47c1.923 7.51 4.403 10.31 4.403 10.69-1.088 2.36-2.343 4.641-3.468 6.983 3.26 4.086 5.06 5.726 8.357 8.356 2.324-1.162 4.644-2.335 6.983-3.467.31 0 3.435 2.545 10.692 4.403l2.47 7.41c4.851.565 7.69.48 11.818 0l2.47-7.41c7.52-1.925 10.297-4.403 10.694-4.403 2.358 1.09 4.64 2.341 6.981 3.467 3.81-3.038 5.522-4.802 8.357-8.356-1.162-2.325-2.335-4.645-3.468-6.984 0-.349 2.55-3.365 4.404-10.692l7.41-2.47c.377-3.24.65-6.236 0-11.816l-7.41-2.477c-2.084-7.972-4.404-10.2-4.404-10.686 1.088-2.36 2.342-4.64 3.467-6.982-3.326-4.246-5.815-6.365-8.358-8.357-2.323 1.161-4.642 2.333-6.98 3.466-.29 0-3.606-2.61-10.693-4.404L50.498.394c-2.366-.276-4.25-.4-6.08-.394zm.253 13.111c16.59 0 30.04 13.45 30.04 30.04S61.261 73.19 44.671 73.19 14.632 59.74 14.632 43.15c0-16.59 13.45-30.039 30.04-30.039z"
            fill="#35d6df"
            transition:fade={{ delay: 900, duration: 2000, easing: cubicIn }}
            class={[
                "opacity-100 drop-shadow-[0_0_6px_#35d6df]",
                cogSpinning && "cog-spin",
            ]}
        />
    {/if}
    {#if heartVisible}
        <path
            d="M26.432.48a27.823 27.823 0 0 0-10.229 1.36C9.583 4.031 3.653 9 1.213 16.684c-2.44 7.685-1.384 17.77 4.863 30.197 6.248 12.427 17.703 27.316 36.54 44.986l1.81 1.7 1.81-1.7c18.837-17.67 30.293-32.559 36.541-44.986 6.248-12.427 7.303-22.512 4.864-30.197C85.2 8.999 79.269 4.03 72.648 1.84 66.028-.35 58.66.04 52.643 2.947c-4.014 1.937-6.532 4.86-8.217 7.557-1.685-2.696-4.203-5.62-8.217-7.557h-.002C33.199 1.494 29.853.668 26.432.481Zm-.293 5.313c2.77.152 5.456.803 7.767 1.92h.002c5.072 2.447 7.32 7.557 8.01 9.608l2.508 7.443 2.508-7.443c.69-2.051 2.937-7.16 8.01-9.608h.001c4.623-2.234 10.745-2.603 16.041-.85 5.297 1.754 9.709 5.434 11.61 11.422 1.9 5.988 1.296 14.602-4.545 26.22-5.63 11.197-16.278 25.14-33.625 41.698C27.08 69.646 16.432 55.702 10.803 44.504c-5.84-11.617-6.446-20.23-4.545-26.219 1.9-5.988 6.31-9.668 11.607-11.422 2.648-.876 5.503-1.222 8.274-1.07Z"
            fill="#ff0202"
            transition:fade={{ duration: 2000, easing: cubicOut }}
            class="opacity-100 drop-shadow-[0_0_6px_#ff0202]"
        />
    {/if}
    <path
        d="m77.345 44.811-58.68-37.83 25.81 79.705 25.67-79.784-58.666 37.734z"
        fill="none"
        stroke="#fff"
        stroke-width="4.498"
        class="opacity-100 drop-shadow-[0.5px_0.5px_1px_#ffffff]"
    />
    {#if eyesVisible}
        <path
            d="M37.69 32.767v5.288m13.964-5.288v5.288"
            fill="none"
            stroke="#35d6df"
            stroke-width="1.323"
            transition:fade={{ delay: 900, duration: 2000, easing: cubicIn }}
            class="opacity-100 drop-shadow-[0_0_4px_#35d6df]"
        />
    {/if}
</svg>

<style>
    .cog-spin {
        transform-origin: 50.3% 46%;
        animation: spin 8s linear infinite;
        animation-fill-mode: forwards;
    }

    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }
</style>
