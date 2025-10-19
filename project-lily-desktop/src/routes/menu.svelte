<script lang="ts">
    import { ArrowLeft, House, Bug, Menu, ThumbsUp, Twitch } from "@lucide/svelte";

    let menuOpen = $state(false);

    const menuItems = [
        {
            name: "Home",
            href: "/",
            icon: House,
        },
        {
            name: "Twitch",
            href: "/twitch",
            icon: Twitch,
        },
        {
            name: "Rewards",
            href: "/rewards",
            icon: ThumbsUp,
        },
        {
            name: "Debug",
            href: "/debug",
            icon: Bug,
        },
    ];

    function width(node: HTMLElement, params: { duration: number }) {
        return {
            duration: params.duration,
            css: (t: number) => `width: ${t * 50}px; overflow: hidden;`,
        };
    }
</script>

<aside class="min-h-full bg-gray-900 shadow-2xl z-50">
    <button
        type="button"
        class="m-4 hover:text-gray-300"
        onclick={() => (menuOpen = !menuOpen)}
    >
        {#if menuOpen}
            <ArrowLeft />
        {:else}
            <Menu />
        {/if}
    </button>
    <nav class="flex flex-col space-y-2">
        {#each menuItems as item}
            {@const Icon = item.icon}
            <a
                href={item.href}
                class="hover:text-gray-300 flex items-center border-b border-gray-700 px-4 py-2"
            >
                <Icon />
                {#if menuOpen}
                    <span class="ml-2" transition:width={{ duration: 300 }}
                        >{item.name}</span
                    >
                {/if}
            </a>
        {/each}
    </nav>
</aside>
