import type { Reward, RewardInstance, StoredReward } from "./rewards/types";
import type { StoredTrigger, Trigger, TriggerInstance } from "./triggers/types";

import { SetAvatarReward } from "./rewards/set-avatar";
import { CancelAvatarReward } from "./rewards/cancel-avatar";
import { AndTrigger } from "./triggers/and";
import { OrTrigger } from "./triggers/or";
import { StreamlabsDonationTrigger } from "./triggers/streamlabs-donation";
import { TwitchBitDonationTrigger } from "./triggers/twitch-bit-donation";
import { TwitchChannelPointsTrigger } from "./triggers/twitch-channel-points";
import { TwitchMessageTrigger } from "./triggers/twitch-message";
import { TwitchWhisperTrigger } from "./triggers/twitch-whisper";
import type { Component } from "svelte";
import SetAvatarEditor from "./components/rewards/set-avatar-editor.svelte";
import StreamlabsDonationEditor from "./components/triggers/streamlabs-donation-editor.svelte";
import TwitchBitDonationEditor from "./components/triggers/twitch-bit-donation-editor.svelte";
import TwitchChannelPointsEditor from "./components/triggers/twitch-channel-points-editor.svelte";
import TwitchMessagishEditor from "./components/triggers/twitch-messagish-editor.svelte";
import { SetOSCReward } from "./rewards/set-osc";
import { CancelOSCReward } from "./rewards/cancel-osc";
import SetOscEditor from "./components/rewards/set-osc-editor.svelte";
import CancelOscEditor from "./components/rewards/cancel-osc-editor.svelte";
import SubtriggerEditor from "./components/triggers/subtrigger-editor.svelte";
import { SetOverlayReward } from "./rewards/set-overlay";
import SetOverlayEditor from "./components/rewards/set-overlay-editor.svelte";
import { CancelOverlayReward } from "./rewards/cancel-overlay";
import CancelOverlayEditor from "./components/rewards/cancel-overlay-editor.svelte";
import { SetWarudoOscReward } from "./rewards/set-warudo-osc";
import SetWarudoOscEditor from "./components/rewards/set-warudo-osc-editor.svelte";

export const rewards: {
    [id: string]: {
        reward: Reward<any>;
        editor?: Component<{ reward: RewardInstance<any> }>
    }
} = {
    [SetAvatarReward.id]: {
        reward: SetAvatarReward,
        editor: SetAvatarEditor,
    },
    [CancelAvatarReward.id]: {
        reward: CancelAvatarReward,
    },
    [SetOSCReward.id]: {
        reward: SetOSCReward,
        editor: SetOscEditor,
    },
    [CancelOSCReward.id]: {
        reward: CancelOSCReward,
        editor: CancelOscEditor,
    },
    [SetOverlayReward.id]: {
        reward: SetOverlayReward,
        editor: SetOverlayEditor
    },
    [CancelOverlayReward.id]: {
        reward: CancelOverlayReward,
        editor: CancelOverlayEditor,
    },
    [SetWarudoOscReward.id]: {
        reward: SetWarudoOscReward,
        editor: SetWarudoOscEditor,
    },
}
export const triggers: {
    [id: string]: {
        trigger: Trigger<any>;
        editor?: Component<{ trigger: TriggerInstance<any> }>
    }
} = {
    [AndTrigger.id]: {
        trigger: AndTrigger,
        editor: SubtriggerEditor,
    },
    [OrTrigger.id]: {
        trigger: OrTrigger,
        editor: SubtriggerEditor,
    },
    [StreamlabsDonationTrigger.id]: {
        trigger: StreamlabsDonationTrigger,
        editor: StreamlabsDonationEditor,
    },
    [TwitchBitDonationTrigger.id]: {
        trigger: TwitchBitDonationTrigger,
        editor: TwitchBitDonationEditor,
    },
    [TwitchChannelPointsTrigger.id]: {
        trigger: TwitchChannelPointsTrigger,
        editor: TwitchChannelPointsEditor,
    },
    [TwitchMessageTrigger.id]: {
        trigger: TwitchMessageTrigger,
        editor: TwitchMessagishEditor,
    },
    [TwitchWhisperTrigger.id]: {
        trigger: TwitchWhisperTrigger,
        editor: TwitchMessagishEditor,
    },
};

export function restoreReward<P>(stored: StoredReward): RewardInstance<P> {
    const reward = rewards[stored.id].reward;

    return new reward(stored.params);
}

export function restoreTrigger(stored: StoredTrigger): TriggerInstance<any> {
    const trigger = triggers[stored.id].trigger;
    return new trigger(stored.params);
}