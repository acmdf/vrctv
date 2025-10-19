import type { PageLoad } from './$types';
import { commands } from '../../bindings';
import { serverConnection } from '$lib/websocket';

export const load: PageLoad = async ({ params }) => {
    return await commands.fetchAvatars();
};