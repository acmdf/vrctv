<script lang="ts">
    import Input from "$lib/components/ui/input/input.svelte";
    import * as InputGroup from "$lib/components/ui/input-group";
    import type { TriggerInstance } from "$lib/triggers/types";
    import { TwitchBitDonationTrigger } from "$lib/triggers/twitch-bit-donation";

    let {
        trigger = $bindable(),
    }: {
        trigger: TriggerInstance<any>;
    } = $props();

    let donationTrigger = $derived.by(() => {
        if (trigger instanceof TwitchBitDonationTrigger) {
            return trigger as TwitchBitDonationTrigger;
        }

        trigger = new TwitchBitDonationTrigger({});
        return trigger as TwitchBitDonationTrigger;
    });
</script>

<div class="grid grid-cols-2 gap-2">
    Above
    <InputGroup.Root>
        <InputGroup.Input
            type="number"
            min="0"
            bind:value={
                () => donationTrigger.params.minimum_amount,
                (v) => {
                    donationTrigger.params.minimum_amount = v;
                    trigger = donationTrigger;
                }
            }
            placeholder="500"
        />
        <InputGroup.Addon align="inline-end"
            ><InputGroup.Text>Bits</InputGroup.Text></InputGroup.Addon
        >
    </InputGroup.Root>
    With text
    <Input
        type="text"
        bind:value={
            () => donationTrigger.params.message_contains,
            (v) => {
                donationTrigger.params.message_contains = v;
                trigger = donationTrigger;
            }
        }
        placeholder="This String"
    />
</div>
