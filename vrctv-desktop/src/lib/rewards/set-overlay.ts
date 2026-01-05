import { get } from "svelte/store";
import { CancellableReward, type RewardContext } from "./types";
import { overlays, overlayVisibleStore } from "$lib/stores/overlays";
import { updateContext } from "$lib/stores/rewards";

export type SetOverlayRewardParams = {
    overlay_id: number;
    timeout_ms: number;
    show: boolean;
}
export class SetOverlayReward extends CancellableReward<SetOverlayRewardParams> {
    static id = "set-overlay-reward";
    static title = "Set Overlay Reward";
    static description = "Set overlay for a duration"
    reward = SetOverlayReward;

    finishTimeout: number | null = null;

    constructor(params: Partial<SetOverlayRewardParams>) {
        super({
            overlay_id: params.overlay_id ?? 0,
            timeout_ms: params.timeout_ms ?? 0,
            show: params.show ?? true,
        });
    }

    async validate(): Promise<string | null> {
        const overlayList = get(overlays);

        if (!overlayList.find((o) => o.id === this.params.overlay_id)) {
            return "Invalid Overlay ID.";
        }

        return null;
    }

    async readyToStart(context: RewardContext): Promise<boolean> {
        let runningReward = context.runningRewards.find((r) => r instanceof SetOverlayReward && r.params.overlay_id === this.params.overlay_id);
        if (runningReward) {
            return false;
        }

        return true;
    }

    async onStart(context: RewardContext): Promise<void> {
        const overlayVisibility = get(overlayVisibleStore);
        overlayVisibility[this.params.overlay_id] = this.params.show;
        overlayVisibleStore.set(overlayVisibility);

        if (this.params.timeout_ms > 0) {
            this.finishTimeout = setTimeout(() => this.onCancel(updateContext(context)), this.params.timeout_ms);
        }
    }

    async isStillRunning(_context: RewardContext): Promise<boolean> {
        return this.finishTimeout !== null;
    }

    async onCancel(_context: RewardContext): Promise<void> {
        if (this.finishTimeout !== null) {
            clearTimeout(this.finishTimeout);
            this.finishTimeout = null;
        }

        const overlayVisibility = get(overlayVisibleStore);
        const overlayList = get(overlays);
        const overlayItem = overlayList.find((o) => o.id === this.params.overlay_id);

        if (!overlayItem) {
            return;
        }

        overlayVisibility[this.params.overlay_id] = overlayItem.visible;
        overlayVisibleStore.set(overlayVisibility);

        this.finishCallback?.();
    }
}