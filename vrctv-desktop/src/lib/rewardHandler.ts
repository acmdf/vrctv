import { get } from "svelte/store";
import { currentReward, overlayVisibleStore, avatarRewardQueue, rewardStore, type AvatarReward, type Reward, type RewardStoreState, overlayRewardQueue, type OverlayReward, overlays, currentAvatarRewardTimeout, currentOverlayRewardTimeout, warudoOscSetRewardTimeouts } from "./stores";
import { commands } from "../bindings";

export async function addReward(reward: Reward) {
    switch (reward.type) {
        case "avatar": {
            const queue = get(avatarRewardQueue);
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
        case "warudoOsc": {
            const queue = get(warudoOscSetRewardTimeouts);

            for (const [address, value] of Object.entries(reward.onStart)) {
                await commands.setWarudoOsc(address, value);
            }

            for (const address of Object.keys(reward.onStop)) {
                if (queue[address]) {
                    clearTimeout(queue[address][0]);
                    finishWarudoOscSetReward(address);
                }

                const timeout = setTimeout(async () => {
                    finishWarudoOscSetReward(address);
                });

                queue[address] = [timeout, reward.onStop[address]];
            }

            warudoOscSetRewardTimeouts.set(queue);
            break;
        }
        case "warudoOscCancel": {
            const queue = get(warudoOscSetRewardTimeouts);

            for (const address of Object.keys(reward.addresses)) {
                if (queue[address]) {
                    clearTimeout(queue[address][0]);
                    finishWarudoOscSetReward(address);
                }
            }

            warudoOscSetRewardTimeouts.set(queue);
            break;
        }
    }
}

async function finishWarudoOscSetReward(address: string) {
    const queue = get(warudoOscSetRewardTimeouts);

    if (!queue[address]) return;

    const value = queue[address][1];
    delete queue[address];

    await commands.setWarudoOsc(address, value);

    warudoOscSetRewardTimeouts.set(queue);
}

export async function cancelReward(avatar: boolean = true, overlay: OverlayReward["overlay"] | null = null) {
    if (avatar) {
        const avatarTimeout = get(currentAvatarRewardTimeout);
        if (avatarTimeout) {
            clearTimeout(avatarTimeout);
            finishAvatarReward();
        }
    }
    if (overlay) {
        const overlayTimeout = get(currentOverlayRewardTimeout);

        if (overlayTimeout[overlay]) {
            clearTimeout(overlayTimeout[overlay]);
            finishOverlayReward(
                overlay
            );
        }
    }
}

async function finishAvatarReward() {
    const queue = get(avatarRewardQueue);
    queue.shift();
    avatarRewardQueue.set(queue);
    if (queue.length > 0) {
        handleAvatarReward(queue[0]);
    } else {
        const baseAvatarId = get(rewardStore).baseAvatarId ?? "";
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

    const timeout = setTimeout(finishAvatarReward, reward.timeoutSeconds * 1000);
    currentAvatarRewardTimeout.set(timeout);
}

async function finishOverlayReward(overlay: OverlayReward["overlay"]) {
    const queue = get(overlayRewardQueue)[overlay];
    queue.shift();
    overlayRewardQueue.update((q) => {
        q[overlay] = queue;
        return q;
    });

    if (queue.length > 0) {
        handleOverlayReward(queue[0]);
    } else {
        const overlayVisibility = get(overlayVisibleStore);
        const overlayList = get(overlays);
        const overlayItem = overlayList.find((o) => o.id === overlay);

        overlayVisibility[overlay] = overlayItem ? overlayItem.visible : false;
        overlayVisibleStore.set(overlayVisibility);
    }
}

async function handleOverlayReward(reward: OverlayReward) {
    const overlayVisibility = get(overlayVisibleStore);
    overlayVisibility[reward.overlay] = reward.show;
    overlayVisibleStore.set(overlayVisibility);

    const timeout = setTimeout(async () => {
        await finishOverlayReward(reward.overlay);
    }, reward.timeoutSeconds * 1000);

    const currentTimeouts = get(currentOverlayRewardTimeout);
    currentTimeouts[reward.overlay] = timeout;
    currentOverlayRewardTimeout.set(currentTimeouts);
}
