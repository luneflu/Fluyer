import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TauriCommands } from '$lib/constants/TauriCommands';

const TauriDeveloperAPI = {
	listenLog: (callback: (event: { payload: string[] }) => void) => {
		return listen<string[]>(TauriCommands.LOG, callback);
	},
	saveDeveloperLog: () => {
		return invoke(TauriCommands.DEVELOPER_LOG_SAVE);
	},
	saveDeveloperMpvLog: () => {
		return invoke(TauriCommands.DEVELOPER_MPV_LOG_SAVE);
	},
	clearDeveloperData: () => {
		return invoke(TauriCommands.DEVELOPER_CLEAR_DATA);
	},
	clearDeveloperCache: () => {
		return invoke(TauriCommands.DEVELOPER_CLEAR_CACHE);
	}
};

export default TauriDeveloperAPI;
