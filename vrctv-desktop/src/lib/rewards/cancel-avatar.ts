import { RewardInstance, type RewardContext } from "./types";
import { SetAvatarReward } from "./set-avatar";

export type CancelAvatarRewardParams = {}
export class CancelAvatarReward extends RewardInstance<CancelAvatarRewardParams> {
    static id = "cancel-avatar-reward";
    static title = "Cancel Avatar Reward";
    static description = "Cancel any active avatar rewards";
    reward = CancelAvatarReward;

    async readyToStart(_context: RewardContext): Promise<boolean> {
        return true;
    }
    async onStart(context: RewardContext): Promise<void> {
        const setAvatarRewards: SetAvatarReward[] = context.runningRewards.filter((r) => r instanceof SetAvatarReward);

        for (const reward of setAvatarRewards) {
            await reward.onCancel(context);
        }
    }
    async isStillRunning(_context: RewardContext): Promise<boolean> {
        return false;
    }
    async validate(): Promise<string | null> {
        return null;
    }
}