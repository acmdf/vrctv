import { get } from "svelte/store";
import type { TwitchEventSource } from "../../../project-lily-common/bindings/TwitchEventSource";
import { currentReward, rewardQueue, rewardStore, type RewardStoreState } from "./stores";
import { commands } from "../bindings";

export async function addReward(reward: RewardStoreState["rewards"][0], event: TwitchEventSource) {
    let queue = get(rewardQueue);
    rewardQueue.update((q) => [...q, reward]);

    if (queue.length > 0) {
        return;
    }

    handleReward(reward, event);
}

async function handleReward(reward: RewardStoreState["rewards"][0], event: TwitchEventSource) {
    await commands.changeAvatar(reward.setsAvatar ?? get(rewardStore).baseAvatarId ?? "");
    currentReward.set(reward);

    setTimeout(async () => {
        let queue = get(rewardQueue);
        queue.shift();
        rewardQueue.set(queue);
        if (queue.length > 0) {
            handleReward(queue[0], event);
        } else {
            let baseAvatarId = get(rewardStore).baseAvatarId ?? "";
            await commands.changeAvatar(baseAvatarId);
            currentReward.set(null);
        }
    }, reward.timeoutSeconds * 1000);
}