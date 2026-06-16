import { invoke } from '@tauri-apps/api/core';
import { TauriCommands } from '$lib/constants/TauriCommands';
import type { MusicData } from '$lib/features/music/types';

export interface LibraryCounts {
	musicCount: number;
	albumCount: number;
}

export interface MusicFilter {
	search: string;
	sortAsc: boolean;
	albumName?: string;
	folderPath?: string;
	playlistPaths?: string[];
}

export enum CollectionType {
	Album = 'album',
	AlbumIndex = 'albumIndex',
	Folder = 'folder',
	Playlist = 'playlist'
}

export type CollectionContext =
	| { type: CollectionType.Album; name: string }
	| { type: CollectionType.AlbumIndex; index: number; search: string; sortAsc: boolean }
	| { type: CollectionType.Folder; path: string }
	| { type: CollectionType.Playlist; paths: string[] };

export interface FolderInfo {
	trackCount: number;
	totalDuration: number;
}

const TauriLibraryAPI = {
	load: async (): Promise<LibraryCounts> => {
		return invoke<LibraryCounts>(TauriCommands.LIBRARY_LOAD);
	},

	getMusicCount: async (filter: MusicFilter): Promise<number> => {
		return invoke<number>(TauriCommands.LIBRARY_MUSIC_COUNT_GET, { filter });
	},

	getMusicByIndex: async (index: number, filter: MusicFilter): Promise<MusicData | null> => {
		return invoke<MusicData | null>(TauriCommands.LIBRARY_MUSIC_GET_BY_INDEX, { index, filter });
	},

	getMusicByPath: async (path: string): Promise<MusicData | null> => {
		return invoke<MusicData | null>(TauriCommands.LIBRARY_MUSIC_GET_BY_PATH, { path });
	},

	getAlbumCount: async (search: string, sortAsc: boolean): Promise<number> => {
		return invoke<number>(TauriCommands.LIBRARY_ALBUM_COUNT_GET, { search, sortAsc });
	},

	getAlbumByIndex: async (
		index: number,
		search: string,
		sortAsc: boolean
	): Promise<MusicData[] | null> => {
		return invoke<MusicData[] | null>(TauriCommands.LIBRARY_ALBUM_GET_BY_INDEX, {
			index,
			search,
			sortAsc
		});
	},

	getAlbumFirstByIndex: async (
		index: number,
		search: string,
		sortAsc: boolean
	): Promise<MusicData | null> => {
		return invoke<MusicData | null>(TauriCommands.LIBRARY_ALBUM_GET_FIRST_BY_INDEX, {
			index,
			search,
			sortAsc
		});
	},

	getQueueCount: async (): Promise<number> => {
		return invoke<number>(TauriCommands.MUSIC_QUEUE_COUNT_GET);
	},

	getQueueByIndex: async (index: number): Promise<MusicData | null> => {
		return invoke<MusicData | null>(TauriCommands.MUSIC_QUEUE_GET_BY_INDEX, { index });
	},

	collectionAddAndPlay: async (context: CollectionContext): Promise<void> => {
		return invoke<void>(TauriCommands.LIBRARY_COLLECTION_ADD_AND_PLAY, { context });
	},

	collectionAddToQueue: async (context: CollectionContext): Promise<void> => {
		return invoke<void>(TauriCommands.LIBRARY_COLLECTION_ADD_TO_QUEUE, { context });
	},

	collectionShuffleAndPlay: async (context: CollectionContext): Promise<void> => {
		return invoke<void>(TauriCommands.LIBRARY_COLLECTION_SHUFFLE_AND_PLAY, { context });
	},

	getFolderInfo: async (path: string): Promise<FolderInfo> => {
		return invoke<FolderInfo>(TauriCommands.LIBRARY_FOLDER_INFO_GET, { path });
	},

	filterFoldersWithMusic: async (paths: string[]): Promise<string[]> => {
		return invoke<string[]>(TauriCommands.LIBRARY_FOLDERS_FILTER_HAS_MUSIC, { paths });
	},

	sync: async (): Promise<void> => {
		return invoke<void>(TauriCommands.LIBRARY_SYNC);
	}
};

export default TauriLibraryAPI;
