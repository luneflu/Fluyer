import TauriFolderAPI from '$lib/tauri/TauriFolderAPI';
import { type FolderData, type MusicData } from '$lib/features/music/types';
import { isWindows } from '$lib/platform';
import folderStore from '$lib/stores/folder.svelte';
import PersistentStoreService from '$lib/services/PersistentStoreService.svelte';
import TauriLibraryAPI from '$lib/tauri/TauriLibraryAPI';

const PATH_SEPARATOR = isWindows() ? '\\' : '/';
const FolderService = {
	PATH_SEPARATOR,

	initialize: async () => {
		FolderService.listenFolderEvents();
	},

	listenFolderEvents: () => {
		$effect(
			() =>
				void (async () => {
					const currentFolder = folderStore.currentFolder;
					let folders: FolderData[];

					if (currentFolder) {
						folders = await TauriFolderAPI.getFolderItems(currentFolder.path);
					} else {
						const musicPaths = await PersistentStoreService.musicPath.get();
						folders = musicPaths.map((path) => ({ path }) as FolderData);
					}

					folders.sort((a, b) => a.path.localeCompare(b.path, undefined, { sensitivity: 'base' }));

					// Pre-filter folders to only those containing music
					const validPaths = await TauriLibraryAPI.filterFoldersWithMusic(
						folders.map((f) => f.path)
					);
					folderStore.list = validPaths.map((path) => ({ path }));
				})()
		);
	},

	navigateToParent: async (folder: FolderData | null) => {
		if (!folder) return;

		const path = folder.path.split(PATH_SEPARATOR).slice(0, -1).join(PATH_SEPARATOR);

		folderStore.currentFolder = { path } as FolderData;
	},

	normalizePath: (path: string): string => {
		if (!isWindows()) return path;
		if (path.substring(1, 4) === ':\\\\') return path;
		return path.replaceAll(':\\', ':\\\\');
	},

	containsMusic: (music: MusicData, folder: FolderData | null): boolean => {
		if (!folder) return false;

		const musicPath = FolderService.normalizePath(music.path);
		const folderPath = FolderService.normalizePath(folder.path);

		if (!musicPath.startsWith(folderPath)) return false;

		const folderPathWithSlash = folderPath.endsWith(PATH_SEPARATOR)
			? folderPath
			: `${folderPath}${PATH_SEPARATOR}`;

		// Music must start with folder path + separator
		if (!musicPath.startsWith(folderPathWithSlash)) return false;

		const remainingPath = musicPath.substring(folderPathWithSlash.length);

		// Check if music is in immediate folder (no nested folders)
		return remainingPath !== '' && !remainingPath.includes(PATH_SEPARATOR);
	},

	containsMusicRecursive: (music: MusicData, folder: FolderData | null): boolean => {
		if (!folder) return false;
		// Normalize paths (fixes double backslashes and case on Windows)
		return FolderService.normalizePath(music.path).startsWith(
			FolderService.normalizePath(folder.path)
		);
	}
};

export default FolderService;
