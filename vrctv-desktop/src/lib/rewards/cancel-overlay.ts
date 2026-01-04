import { RewardInstance, type RewardContext } from "./types";
import { SetOverlayReward } from "./set-overlay";

export type CancelOverlayRewardParams = {
    overlay_id?: number;
}
export class CancelOverlayReward extends RewardInstance<CancelOverlayRewardParams> {
    static id = "cancel-overlay-reward";
    static title = "Cancel Overlay Reward";
    static description = "Cancel any active overlay rewards (optionally for a specific overlay)";
    reward = CancelOverlayReward;
    
    async readyToStart(_context: RewardContext): Promise<boolean> {
        return true;
    }
    async onStart(context: RewardContext): Promise<void> {
        const setOverlayRewards: SetOverlayReward[] = context.runningRewards.filter((r) => r instanceof SetOverlayReward);

        for (const reward of setOverlayRewards) {
            if (!this.params.overlay_id || reward.params.overlay_id === this.params.overlay_id)
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