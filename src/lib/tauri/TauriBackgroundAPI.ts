import { invoke } from '@tauri-apps/api/core';
import { TauriCommands } from '$lib/constants/TauriCommands';
import { listen } from '@tauri-apps/api/event';

const TauriBackgroundAPI = {
	isCefEnabled: () => {
		return invoke<boolean>(TauriCommands.IS_CEF_ENABLED);
	},
	updateBackground: (
		colors: { r: number; g: number; b: number }[] | null,
		width: number,
		height: number
	): Promise<[number[], number, number] | null> => {
		return invoke(TauriCommands.ANIMATED_BACKGROUND_UPDATE, { colors, width, height });
	},
	restoreBackground: () => {
		return invoke(TauriCommands.ANIMATED_BACKGROUND_RESTORE);
	},
	listenTransitionComplete(callback: () => void) {
		return listen(TauriCommands.ANIMATED_BACKGROUND_TRANSITION_COMPLETE, callback);
	}
};

export default TauriBackgroundAPI;
