import { get } from "svelte/store";
import { currentReward, overlayVisibleStore, avatarRewardQueue, rewardStore, type AvatarReward, type Reward, type RewardStoreState, overlayRewardQueue, type OverlayReward, overlays, currentAvatarRewardTimeout, currentOverlayRewardTimeout } from "./stores";
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
        case "avatarCancel": {
            await cancelReward(true, null);
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
        case "overlayCancel": {
            await cancelReward(false, reward.overlay);
            break;
        }
    }
}

export async function cancelReward(avatar: boolean = true, overlay: OverlayReward["overlay"] | null = null) {
    if (avatar) {
        let avatarTimeout = get(currentAvatarRewardTimeout);
        if (avatarTimeout) {
            clearTimeout(avatarTimeout);
            finishAvatarReward();
        }
    }
    if (overlay) {
        let overlayTimeout = get(currentOverlayRewardTimeout);

        if (overlayTimeout[overlay]) {
            clearTimeout(overlayTimeout[overlay]);
            finishOverlayReward(
                overlay
            );
        }
    }
}

async function finishAvatarReward() {
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
}


async function handleAvatarReward(reward: AvatarReward) {
    await commands.changeAvatar(reward.setsAvatar ?? get(rewardStore).baseAvatarId ?? "");
    for (const [key, value] of Object.entries(reward.setParams ?? {})) {
        await commands.setOsc(key, value);
    }

    currentReward.set(reward);

    let timeout = setTimeout(finishAvatarReward, reward.timeoutSeconds * 1000);
    currentAvatarRewardTimeout.set(timeout);
}

async function finishOverlayReward(overlay: OverlayReward["overlay"]) {
    let queue = get(overlayRewardQueue)[overlay];
    queue.shift();
    overlayRewardQueue.update((q) => {
        q[overlay] = queue;
        return q;
    });

    if (queue.length > 0) {
        handleOverlayReward(queue[0]);
    } else {
        let overlayVisibility = get(overlayVisibleStore);
        let overlayList = get(overlays);
        let overlayItem = overlayList.find((o) => o.id === overlay);

        overlayVisibility[overlay] = overlayItem ? overlayItem.visible : false;
        overlayVisibleStore.set(overlayVisibility);
    }
}


async function handleOverlayReward(reward: OverlayReward) {
    let overlayVisibility = get(overlayVisibleStore);
    overlayVisibility[reward.overlay] = reward.show;
    overlayVisibleStore.set(overlayVisibility);

    let timeout = setTimeout(async () => {
        await finishOverlayReward(reward.overlay);
    }, reward.timeoutSeconds * 1000);

    let currentTimeouts = get(currentOverlayRewardTimeout);
    currentTimeouts[reward.overlay] = timeout;
    currentOverlayRewardTimeout.set(currentTimeouts);
}
