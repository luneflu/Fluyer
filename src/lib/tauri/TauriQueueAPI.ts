import { invoke } from '@tauri-apps/api/core';
import { TauriCommands } from '$lib/constants/TauriCommands';
import type { MusicData } from '$lib/features/music/types';
const TauriQueueAPI = {
	goTo: (index: number) => {
		return invoke(TauriCommands.MUSIC_QUEUE_GOTO, { index });
	},
	add: (list: MusicData[]) => {
		return invoke(TauriCommands.MUSIC_QUEUE_ADD, {
			paths: list.map((m) => m.path)
		});
	},
	remove: (index: number) => {
		return invoke(TauriCommands.MUSIC_QUEUE_REMOVE, {
			index
		});
	},
	moveTo: (from: number, to: number) => {
		return invoke(TauriCommands.MUSIC_QUEUE_MOVETO, {
			from,
			to
		});
	}
};

export default TauriQueueAPI;
