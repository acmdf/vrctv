<script lang="ts">
    import Input from "$lib/components/ui/input/input.svelte";
    import * as InputGroup from "$lib/components/ui/input-group";
    import { StreamlabsDonationTrigger } from "$lib/triggers/streamlabs-donation";
    import type { TriggerInstance } from "$lib/triggers/types";

    let {
        trigger = $bindable(),
    }: {
        trigger: TriggerInstance<any>;
    } = $props();

    let donationTrigger = $derived.by(() => {
        if (trigger instanceof StreamlabsDonationTrigger) {
            return trigger as StreamlabsDonationTrigger;
        }

        trigger = new StreamlabsDonationTrigger({});
        return trigger as StreamlabsDonationTrigger;
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
            placeholder="Minimum Amount"
        />
        <InputGroup.Addon align="inline-end"
            ><InputGroup.Text>In Local Currency</InputGroup.Text
            ></InputGroup.Addon
        >
    </InputGroup.Root>
    Containing Message
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
