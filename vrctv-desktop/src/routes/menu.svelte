<script lang="ts">
    import { House, Bug, ThumbsUp, Twitch, BookOpen } from "@lucide/svelte";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";

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
            name: "Overlays",
            href: "/overlays",
            icon: BookOpen,
        },
        {
            name: "Debug",
            href: "/debug",
            icon: Bug,
        },
    ];
</script>

<Sidebar.Root collapsible="icon">
    <Sidebar.Content>
        <Sidebar.Group>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each menuItems as item (item.name)}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton>
                                {#snippet child({ props })}
                                    <a href={item.href} {...props}>
                                        <item.icon />
                                        <span
                                            class="group-data-[collapsible=icon]:hidden"
                                            >{item.name}</span
                                        >
                                    </a>
                                {/snippet}
                            </Sidebar.MenuButton>
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>
    </Sidebar.Content>
</Sidebar.Root>
