import { get } from "svelte/store";
import type { TwitchEventSource } from "../../../project-lily-common/bindings/TwitchEventSource";
import { rewardStore } from "./stores";
import { addReward } from "./rewardHandler";

export async function handleTwitchEvent(event: TwitchEventSource) {
    let rewards = get(rewardStore).rewards.filter((r) => r.on.type === "twitch");

    for (let reward of rewards) {
        let matches = reward.on.matches as Partial<TwitchEventSource>[];

        for (let m of matches) {
            if (m.type !== event.type) {
                continue;
            }

            switch (m.type) {
                case "ChannelPoints":
                    event = event as Extract<TwitchEventSource, { type: "ChannelPoints" }>;
                    if (m.reward_id && m.reward_id !== event.reward_id) {
                        continue;
                    }
                    break;
                case "BitDonation":
                    event = event as Extract<TwitchEventSource, { type: "BitDonation" }>;
                    if (m.amount && event.amount < m.amount) {
                        continue;
                    }
                    if (m.message && !event.message?.includes(m.message)) {
                        continue;
                    }
                    break;
                case "Message":
                    event = event as Extract<TwitchEventSource, { type: "Message" }>;
                    if (m.message && !event.message.includes(m.message)) {
                        continue;
                    }
                    break;
                case "Whisper":
                    event = event as Extract<TwitchEventSource, { type: "Whisper" }>;
                    if (m.message && !event.message.includes(m.message)) {
                        continue;
                    }
                    break;
            }

            console.log(`Matched reward ${reward.title} for event ${JSON.stringify(event)}`);
            addReward(reward, event);
        }
    }
}
