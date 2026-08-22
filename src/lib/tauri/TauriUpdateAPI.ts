import { invoke } from '@tauri-apps/api/core';
import { TauriCommands } from '$lib/constants/TauriCommands';

const TauriUpdateAPI = {
	checkUpdate: (currentVersion: string) => {
		return invoke<string | null>(TauriCommands.DEVELOPER_UPDATE_CHECK, { currentVersion });
	}
};

export default TauriUpdateAPI;
