import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TauriCommands } from '$lib/constants/TauriCommands';

const TauriDeveloperAPI = {
	listenLog: (callback: (event: { payload: string[] }) => void) => {
		return listen<string[]>(TauriCommands.LOG, callback);
	},
	getDeveloperLog: () => {
		return invoke<[string, string][]>(TauriCommands.DEVELOPER_LOG_GET);
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
	},
	getDeveloperMetrics: () => {
		return invoke<{
			total_ram_bytes: number;
			total_app_ram_bytes: number;
			total_app_working_set_bytes: number;
			total_app_private_ws_bytes: number;
			total_app_cpu_percent: number;
			total_app_gpu_percent: number;
			processes: {
				pid: number;
				name: string;
				is_main: boolean;
				ram_bytes: number;
				working_set_bytes: number;
				private_ws_bytes: number;
				cpu_percent: number;
				gpu_percent: number;
			}[];
		}>(TauriCommands.DEVELOPER_METRICS_GET);
	}
};

export default TauriDeveloperAPI;
