import PersistentStoreService from '$lib/services/PersistentStoreService.svelte';
import FolderService from '$lib/services/FolderService.svelte';
import { MusicConfig } from '$lib/constants/MusicConfig';
import ProgressService from '$lib/services/ProgressService.svelte';
import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';
import ToastService from '$lib/services/ToastService.svelte';
import TauriLibraryAPI, {
	CollectionType,
	type CollectionContext
} from '$lib/tauri/TauriLibraryAPI';
import filterStore from '$lib/stores/filter.svelte';
import musicStore from '$lib/stores/music.svelte';
import { MusicListType } from '$lib/features/music/types';
import folderStore from '$lib/stores/folder.svelte';
import playlistStore from '$lib/stores/playlist.svelte';
import PlaylistService from '$lib/services/PlaylistService.svelte';

const album = $derived(filterStore.album);
const showBackButton = $derived.by(async () => {
	const isNotFolderView = musicStore.listType !== MusicListType.Folder;

	const folderPath = folderStore.currentFolder?.path;
	const storedPath = await PersistentStoreService.musicPath.get();
	const isOutsideStoredPath = folderPath ? !storedPath.includes(folderPath) : false;

	return (
		isNotFolderView || (storedPath.length === 1 && isOutsideStoredPath) || storedPath.length > 1
	);
});

function buildContext(): CollectionContext | null {
	if (musicStore.listType === MusicListType.Playlist && playlistStore.selectedPlaylist) {
		return { type: CollectionType.Playlist, paths: playlistStore.selectedPlaylist.paths };
	}
	if (musicStore.listType === MusicListType.Folder && folderStore.currentFolder) {
		return { type: CollectionType.Folder, path: folderStore.currentFolder.path };
	}
	if (album) {
		return { type: CollectionType.Album, name: album.name };
	}
	return null;
}

const label = $derived.by(() => {
	if (musicStore.listType === MusicListType.Playlist && playlistStore.selectedPlaylist) {
		const pl = playlistStore.selectedPlaylist;
		return [pl.title || pl.name, pl.artist, `${pl.paths.length} tracks`]
			.filter(Boolean)
			.join(` ${MusicConfig.separator} `);
	} else if (musicStore.listType === MusicListType.Folder && folderStore.currentFolder) {
		return folderStore.currentFolder.path;
	} else if (album) {
		return [album.name, album.artist, album.year, album.duration]
			.filter(Boolean)
			.join(` ${MusicConfig.separator} `);
	}
	return null;
});

async function handleBack() {
	if (musicStore.listType === MusicListType.Playlist) {
		playlistStore.selectedPlaylist = null;
		return;
	}
	if (musicStore.listType === MusicListType.Folder) {
		const musicPaths = await PersistentStoreService.musicPath.get();
		if (musicPaths.includes(folderStore.currentFolder!.path)) {
			folderStore.currentFolder = null;
			return;
		}
		await FolderService.navigateToParent(folderStore.currentFolder);
	} else {
		filterStore.album = null;
	}
}

async function addMusicListAndPlay() {
	const context = buildContext();
	if (!context) return;

	await TauriLibraryAPI.collectionAddAndPlay(context);
	if (!musicStore.isPlaying) MusicPlayerService.play();
}

async function addMusicList() {
	const context = buildContext();
	if (!context) return;

	await TauriLibraryAPI.collectionAddToQueue(context);

	const toastLabel =
		musicStore.listType === MusicListType.Folder && folderStore.currentFolder
			? folderStore.currentFolder.path
			: album
				? `${album.name} ${MusicConfig.separatorAlbum} ${album.artist}`
				: null;
	ToastService.info(`Added music list to queue: ${toastLabel}`);
}

async function playShuffle() {
	await MusicPlayerService.pause();

	const context = buildContext();
	if (!context) return;

	await TauriLibraryAPI.collectionShuffleAndPlay(context);

	ProgressService.start();
}

async function deletePlaylist() {
	if (playlistStore.selectedPlaylist && playlistStore.selectedPlaylist.id) {
		PlaylistService.delete(playlistStore.selectedPlaylist.id);
		playlistStore.selectedPlaylist = null;
	}
}

export default function useCollectionInfo() {
	return {
		get album() {
			return album;
		},
		get showBackButton() {
			return showBackButton;
		},
		get label() {
			return label;
		},

		handleBack,
		addMusicListAndPlay,
		addMusicList,
		playShuffle,
		deletePlaylist
	};
}
