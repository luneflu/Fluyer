import TauriDeveloperAPI from '$lib/tauri/TauriDeveloperAPI';
import PersistentStoreService from '$lib/services/PersistentStoreService.svelte';
import ToastService from '$lib/services/ToastService.svelte';
import { untrack } from 'svelte';

enum Level {
	Error = 1,
	Warn = 2,
	Info = 3,
	Debug = 4,
	Trace = 5
}

export const logs: { level: string; message: string; timestamp: Date }[] = $state([]);

const LogService = {
	initialize: async () => {
		LogService.listenLog();
		await LogService.fetchInitialBackendLogs();
		LogService.listenBackendLog();
	},
	fetchInitialBackendLogs: async () => {
		try {
			const initialLogs = await TauriDeveloperAPI.getDeveloperLog();
			for (const [levelStr, message] of initialLogs) {
				LogService.processBackendLog(levelStr, message);
			}
		} catch (e) {
			console.error("Failed to fetch initial backend logs", e);
		}
	},
	processBackendLog: (levelStr: string, message: string) => {
		// For logcat (Android)
		if (levelStr === 'ADBLOG') {
			logs.push({ level: 'ADBLOG', message, timestamp: new Date() });
			return;
		}
		
		// Rust logger sends strings like 'INFO', 'ERROR', not numbers
		switch (levelStr) {
			case 'ERROR':
				logs.push({ level: 'RS-ERROR', message, timestamp: new Date() });
				break;
			case 'WARN':
				logs.push({ level: 'RS-WARN', message, timestamp: new Date() });
				break;
			case 'INFO':
				logs.push({ level: 'RS-INFO', message, timestamp: new Date() });
				break;
			case 'DEBUG':
				logs.push({ level: 'RS-DEBUG', message, timestamp: new Date() });
				break;
			case 'TRACE':
				logs.push({ level: 'RS-TRACE', message, timestamp: new Date() });
				break;
			default:
				logs.push({ level: 'RS-LOG', message: `${levelStr}: ${message}`, timestamp: new Date() });
		}
		
		if (logs.length > 1000) {
			logs.splice(0, logs.length - 1000); // Keep last 1000
		}
	},
	listenLog: () => {
		const methods = ['log', 'trace', 'debug', 'info', 'warn', 'error'] as const;
		let isLogging = false;

		methods.forEach((method) => {
			const original = console[method];
			console[method] = (...args: any[]) => {
				if (isLogging) return original.apply(console, args);
				
				isLogging = true;
				try {
					const msg = args.join(' ');
					untrack(() => logs.push({ level: 'WEB-' + method.toUpperCase(), message: msg, timestamp: new Date() }));
				} finally {
					isLogging = false;
				}
				
				return original.apply(console, args);
			};
		});
	},
	listenBackendLog: () => {
		TauriDeveloperAPI.listenLog((event) => {
			const levelStr = event.payload[0];
			const message = event.payload[1];
			LogService.processBackendLog(levelStr, message);
		});
	},
};

export default LogService;
