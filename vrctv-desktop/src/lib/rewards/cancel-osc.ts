import { RewardInstance, type RewardContext } from "./types";
import { SetOSCReward } from "./set-osc";

export type CancelOSCRewardParams = {
    id: string;
    channel_id?: string;
}
export class CancelOSCReward extends RewardInstance<CancelOSCRewardParams> {
    static id = "cancel-osc-reward";
    static title = "Cancel OSC Reward";
    static description = "Cancel any active OSC rewards (optionally for a specific channel)";
    reward = CancelOSCReward;

    async readyToStart(_context: RewardContext): Promise<boolean> {
        return true;
    }
    async onStart(context: RewardContext): Promise<void> {
        const setOscRewards: SetOSCReward[] = context.runningRewards.filter((r) => r instanceof SetOSCReward);

        for (const reward of setOscRewards) {
            if (!this.params.channel_id || reward.params.channel_id === this.params.channel_id)
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