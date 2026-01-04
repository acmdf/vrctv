export interface DonationMessage {
    _id: string;
    amount: number;
    currency: string;
    formattedAmount: string;
    from: string;
    from_user_id: number;
    isPreview: boolean;
    isTest: boolean;
    message: string;
    name: string;
    priority: number;
    recurring: boolean;
    to: {
        name: string;
    };
    unsavedSettings: unknown[];
}

export type KnownStreamlabsEvents = { "type": "donation"; amount: number; message: DonationMessage[] };