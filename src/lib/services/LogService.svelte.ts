import TauriDeveloperAPI from '$lib/tauri/TauriDeveloperAPI';
import PersistentStoreService from '$lib/services/PersistentStoreService.svelte';
import ToastService from '$lib/services/ToastService.svelte';

enum Level {
	Error = 1,
	Warn = 2,
	Info = 3,
	Debug = 4,
	Trace = 5
}
const LogService = {
	initialize: async () => {
		LogService.listenLog();
		LogService.listenError();
		LogService.listenBackendLog();
	},
	listenLog: () => {
		console.log = new Proxy(console.log, {
			apply(target, thisArg, args) {
				return Reflect.apply(target, thisArg, ['[LOG]', ...args]);
			}
		});
		console.trace = new Proxy(console.trace, {
			apply(target, thisArg, args) {
				return Reflect.apply(target, thisArg, ['[TRACE]', ...args]);
			}
		});
		console.debug = new Proxy(console.debug, {
			apply(target, thisArg, args) {
				return Reflect.apply(target, thisArg, ['[DEBUG]', ...args]);
			}
		});
		console.info = new Proxy(console.info, {
			apply(target, thisArg, args) {
				return Reflect.apply(target, thisArg, ['[INFO]', ...args]);
			}
		});
		console.warn = new Proxy(console.warn, {
			apply(target, thisArg, args) {
				return Reflect.apply(target, thisArg, ['[WARN]', ...args]);
			}
		});
		console.error = new Proxy(console.error, {
			apply(target, thisArg, args) {
				return Reflect.apply(target, thisArg, ['[ERROR]', ...args]);
			}
		});
	},
	listenBackendLog: () => {
		TauriDeveloperAPI.listenLog((event) => {
			switch (parseInt(event.payload[0])) {
				case Level.Error:
					console.error(event.payload[1]);
					break;
				case Level.Warn:
					console.warn(event.payload[1]);
					break;
				case Level.Info:
					console.info(event.payload[1]);
					break;
				case Level.Debug:
					console.debug(event.payload[1]);
					break;
				case Level.Trace:
					console.trace(event.payload[1]);
					break;
			}
		});
	},
	listenError: () => {
		window.addEventListener('error', async (e) => {
			if (e.message.toString().includes('ResizeObserver')) return;
			if (await PersistentStoreService.developerMode.get())
				ToastService.error(e.message.toString());
		});
		window.addEventListener('unhandledrejection', async (e) => {
			const message = e.reason.toString();
			if (await PersistentStoreService.developerMode.get()) ToastService.error(message);
		});
	}
};

export default LogService;
