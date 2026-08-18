import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TauriCommands } from '$lib/constants/TauriCommands';
import type { MusicPlayerSync, RepeatMode } from '$lib/features/music/types';

const TauriMusicAPI = {
	play: () => {
		return invoke(TauriCommands.MUSIC_PLAY);
	},
	pause: () => {
		return invoke(TauriCommands.MUSIC_PAUSE);
	},
	next: () => {
		return invoke(TauriCommands.MUSIC_NEXT);
	},
	previous: () => {
		return invoke(TauriCommands.MUSIC_PREVIOUS);
	},
	clear: () => {
		return invoke(TauriCommands.MUSIC_CLEAR);
	},
	setRepeatMode: (mode: RepeatMode) => {
		return invoke(TauriCommands.MUSIC_REPEAT_MODE_SET, { mode });
	},
	setPosition: (position: number) => {
		return invoke(TauriCommands.MUSIC_POSITION_SET, {
			position: Math.trunc(position)
		});
	},
	requestSync: () => {
		return invoke(TauriCommands.MUSIC_PLAYER_REQUEST_SYNC);
	},
	setVolume: (volume: number) => {
		return invoke(TauriCommands.MUSIC_VOLUME_SET, { volume });
	},
	setDiscordRpcEnabled: (enabled: boolean) => {
		return invoke(TauriCommands.MUSIC_DISCORD_RPC_SET, { enabled });
	},
	listenSync: (callback: (event: { payload: MusicPlayerSync }) => void) => {
		return listen<MusicPlayerSync>(TauriCommands.MUSIC_PLAYER_SYNC, callback);
	}
};

export default TauriMusicAPI;
