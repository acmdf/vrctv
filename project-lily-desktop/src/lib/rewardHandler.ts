import { get } from "svelte/store";
import { currentReward, overlayVisibleStore, avatarRewardQueue, rewardStore, type AvatarReward, type Reward, type RewardStoreState, overlayRewardQueue, type OverlayReward, overlays } from "./stores";
import { commands } from "../bindings";

export async function addReward(reward: Reward) {
    switch (reward.type) {
        case "avatar": {
            let queue = get(avatarRewardQueue);
            avatarRewardQueue.update((q) => [...q, reward]);

            if (queue.length > 0) {
                return;
            }

            handleAvatarReward(reward);
            break;
        }
        case "overlay": {
            let queue = get(overlayRewardQueue)[reward.overlay];

            if (!queue) {
                overlayRewardQueue.update((q) => {
                    q[reward.overlay] = [];
                    return q;
                });
                queue = [];
            }

            overlayRewardQueue.update((q) => {
                q[reward.overlay] = [...queue, reward];
                return q;
            });

            if (queue.length > 0) {
                return;
            }

            handleOverlayReward(reward);
            break;
        }
    }
}

async function handleAvatarReward(reward: AvatarReward) {
    await commands.changeAvatar(reward.setsAvatar ?? get(rewardStore).baseAvatarId ?? "");
    for (const [key, value] of Object.entries(reward.setParams ?? {})) {
        await commands.setOsc(key, value);
    }

    currentReward.set(reward);

    setTimeout(async () => {
        let queue = get(avatarRewardQueue);
        queue.shift();
        avatarRewardQueue.set(queue);
        if (queue.length > 0) {
            handleAvatarReward(queue[0]);
        } else {
            let baseAvatarId = get(rewardStore).baseAvatarId ?? "";
            await commands.changeAvatar(baseAvatarId);
            for (const [key, value] of Object.entries(get(rewardStore).baseParams ?? {})) {
                await commands.setOsc(key, value);
            }
            currentReward.set(null);
        }
    }, reward.timeoutSeconds * 1000);
}

async function handleOverlayReward(reward: OverlayReward) {
    let overlayVisibility = get(overlayVisibleStore);
    overlayVisibility[reward.overlay] = reward.show;
    overlayVisibleStore.set(overlayVisibility);

    setTimeout(async () => {
        let queue = get(overlayRewardQueue)[reward.overlay];
        queue.shift();
        overlayRewardQueue.update((q) => {
            q[reward.overlay] = queue;
            return q;
        });

        if (queue.length > 0) {
            handleOverlayReward(queue[0]);
        } else {
            let overlayVisibility = get(overlayVisibleStore);
            let overlayList = get(overlays);
            let overlayItem = overlayList.find((o) => o.id === reward.overlay);

            overlayVisibility[reward.overlay] = overlayItem ? overlayItem.visible : false;
            overlayVisibleStore.set(overlayVisibility);
        }
    }, reward.timeoutSeconds * 1000);
}
