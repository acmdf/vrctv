import type { PageLoad } from './$types';
import { commands, type Avatar, type Result } from '../../bindings';

export const load = async ({ params }) => {
    return await commands.fetchAvatars();
};