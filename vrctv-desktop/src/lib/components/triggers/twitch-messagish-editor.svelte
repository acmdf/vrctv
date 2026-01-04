<script lang="ts">
    import Input from "$lib/components/ui/input/input.svelte";
    import * as InputGroup from "$lib/components/ui/input-group";
    import type { TriggerInstance } from "$lib/triggers/types";
    import { TwitchWhisperTrigger } from "$lib/triggers/twitch-whisper";
    import { TwitchMessageTrigger } from "$lib/triggers/twitch-message";

    let {
        trigger = $bindable(),
    }: {
        trigger: TriggerInstance<any>;
    } = $props();

    let messageTrigger = $derived.by(() => {
        if (
            trigger instanceof TwitchMessageTrigger ||
            trigger instanceof TwitchWhisperTrigger
        ) {
            return trigger as TwitchMessageTrigger;
        }

        trigger = new TwitchMessageTrigger({});
        return trigger as TwitchMessageTrigger;
    });
</script>

<div class="grid grid-cols-2 gap-2">
    Containing
    <Input
        type="text"
        bind:value={
            () => messageTrigger.params.message_contains,
            (v) => {
                messageTrigger.params.message_contains = v;
                trigger = messageTrigger;
            }
        }
        placeholder="This String"
    />
    From
    <Input
        type="text"
        bind:value={
            () => messageTrigger.params.sender,
            (v) => {
                messageTrigger.params.sender = v;
                trigger = messageTrigger;
            }
        }
        placeholder="someuser"
    />
</div>
