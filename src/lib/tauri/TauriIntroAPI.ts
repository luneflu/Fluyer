import { invoke } from '@tauri-apps/api/core';
import { TauriCommands } from '$lib/constants/TauriCommands';
import { isAndroid } from '$lib/platform';
import { listen } from '@tauri-apps/api/event';

const TauriIntroAPI = {
	requestReadAudioPermission: async () => {
		if (!isAndroid()) return false;
		return await invoke<boolean>(TauriCommands.MOBILE_AUDIO_PERMISSION_READ_REQUEST);
	},
	requestDirectoryPath: () => {
		return new Promise<string>(async (resolve, _) => {
			const command = isAndroid()
				? TauriCommands.MOBILE_ANDROID_DIRECTORY_REQUEST
				: TauriCommands.MUSIC_DIRECTORY_REQUEST;

			await invoke(command);

			const unlisten = await listen<string>(command, (e) => {
				resolve(e.payload);
				unlisten();
			});
		});
	}
};

export default TauriIntroAPI;
