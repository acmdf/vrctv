import { cachedAvatarStore } from "$lib/avatar-list-cache";
import { get } from "svelte/store";
import { commands } from "../../bindings";
import { CancellableReward, type RewardContext } from "./types";
import { oscStateStore } from "$lib/stores/global";
import { info } from "@tauri-apps/plugin-log";
import { rewardStore, updateContext } from "$lib/stores/rewards";

export type SetAvatarRewardParams = {
    avatar_id: string;
    return_to: "default" | "previous" | "specific";
    return_avatar_id?: string;
    timeout_ms: number;
}
export class SetAvatarReward extends CancellableReward<SetAvatarRewardParams> {
    static id = "set-avatar-reward";
    static title = "Set Avatar Reward";
    static description = "Set avatar for a duration"
    reward = SetAvatarReward;

    finishTimeout: number | null = null;
    caughtPreviousAvatarId: string | null = null;

    constructor(params: Partial<SetAvatarRewardParams>) {
        super({
            avatar_id: params.avatar_id ?? "",
            return_to: params.return_to ?? "default",
            timeout_ms: params.timeout_ms ?? 0,
            return_avatar_id: params.return_avatar_id,
        });
    }

    async validate(): Promise<string | null> {
        if (!this.params.avatar_id) {
            return "Avatar ID cannot be empty.";
        }

        const avatarStore = get(cachedAvatarStore);

        if (avatarStore.find((a) => a.id === this.params.avatar_id) === undefined) {
            return "Invalid Avatar ID.";
        }

        if (this.params.return_avatar_id && avatarStore.find((a) => a.id === this.params.return_avatar_id) === undefined) {
            return "Invalid Return Avatar ID.";
        }

        return null;
    }

    async readyToStart(context: RewardContext): Promise<boolean> {
        let runningReward = context.runningRewards.find((r) => r instanceof SetAvatarReward)
        if (runningReward) {
            if (!this.caughtPreviousAvatarId && runningReward.params.return_to === "previous") {
                // It will return to the previous avatar, which works if there isn't a queued SetAvatarReward, but if there is, it will try to return to that one instead, so we catch the previous avatar ID from this one
                this.caughtPreviousAvatarId = runningReward.params.return_avatar_id || null;
            }

            return false;
        }

        return true;
    }

    async onStart(context: RewardContext): Promise<void> {
        if (this.params.return_to === "previous") {
            if (this.caughtPreviousAvatarId) { // We caught it in readyToStart when we were added to the queue
                this.params.return_avatar_id = this.caughtPreviousAvatarId;
                this.caughtPreviousAvatarId = null;
            } else { // We are starting immediately, so get the current avatar ID from OSC
                const currentAvatarId = get(oscStateStore)["/avatar/change"];

                if (currentAvatarId && "String" in currentAvatarId) {
                    this.params.return_avatar_id = currentAvatarId.String;
                } else {
                    info("SetAvatarReward: No current avatar ID found in OSC state store.");
                    this.params.return_avatar_id = undefined;
                }
            }
        } else if (this.params.return_to === "default") {
            this.params.return_avatar_id = get(rewardStore).baseAvatarId;
        }

        await commands.changeAvatar(this.params.avatar_id);
        if (this.params.timeout_ms > 0) {
            this.finishTimeout = setTimeout(() => this.onCancel(updateContext(context)), this.params.timeout_ms);
        }
    }

    async isStillRunning(_context: RewardContext): Promise<boolean> {
        return this.finishTimeout !== null;
    }

    async onCancel(context: RewardContext): Promise<void> {
        if (this.finishTimeout !== null) {
            clearTimeout(this.finishTimeout);
            this.finishTimeout = null;
        }

        // First check that there isn't another queued SetAvatarReward, which would override this one and make returning to the previous avatar incorrect
        const runningSetAvatarReward = context.rewardQueue.find((r) => r instanceof SetAvatarReward) as SetAvatarReward | undefined;
        if (runningSetAvatarReward) {
            // If there is, do not change the avatar back
            this.finishCallback?.();
            return;
        }

        if (this.params.return_avatar_id) {
            await commands.changeAvatar(this.params.return_avatar_id);
        }

        this.finishCallback?.();
    }
}