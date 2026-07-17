import { type FolderData, type MusicData, MusicListType } from '../types';
import filterStore from '$lib/stores/filter.svelte';
import filterBarStore from '$lib/stores/filterBar.svelte';
import playlistStore from '$lib/stores/playlist.svelte';
import MetadataService from '$lib/services/MetadataService.svelte';
import folderStore from '$lib/stores/folder.svelte';
import FolderService from '$lib/services/FolderService.svelte';
import musicStore from '$lib/stores/music.svelte';
import { MusicConfig } from '$lib/constants/MusicConfig';
import ProgressService from '$lib/services/ProgressService.svelte';
import QueueService from '$lib/services/QueueService.svelte';
import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';
import ToastService from '$lib/services/ToastService.svelte';
import { COVER_ART_DEBOUNCE_DELAY, CoverArtSize } from '$lib/services/CoverArtService.svelte';
import TauriLibraryAPI, { CollectionType, type FolderInfo } from '$lib/tauri/TauriLibraryAPI';

export function useMusicItem(
	getMusicIndex: () => number | undefined,
	getMusicProp: () => MusicData | undefined,
	getFolder: () => FolderData | undefined,
	getVisible: () => boolean = () => true
) {
	let coverArt = $state<Promise<string | null> | null>(null);
	let currentBlobUrl: string | null = null;
	let folderInfo = $state<FolderInfo | null>(null);

	const musicIndex = $derived(getMusicIndex());
	const musicProp = $derived(getMusicProp());
	const folder = $derived(getFolder());
	const visible = $derived(getVisible());

	let resolvedMusic = $state<MusicData | undefined>(musicProp);

	$effect(() => {
		if (musicProp !== undefined) {
			resolvedMusic = musicProp;
		}
	});

	$effect(() => {
		if (musicIndex === undefined || !visible) return;

		const isFolderMode = musicStore.listType === MusicListType.Folder;
		const isPlaylistMode = musicStore.listType === MusicListType.Playlist;

		const filter = {
			search: filterStore.search,
			sortAsc: filterBarStore.sortAsc,
			albumName: filterStore.album?.name,
			folderPath:
				isFolderMode && folderStore.currentFolder ? folderStore.currentFolder.path : undefined,
			playlistPaths:
				isPlaylistMode && playlistStore.selectedPlaylist
					? playlistStore.selectedPlaylist.paths
					: undefined
		};

		TauriLibraryAPI.getMusicByIndex(musicIndex, filter).then((m) => {
			if (m) resolvedMusic = m;
		});
	});

	const music = $derived(resolvedMusic);

	const isSelectedForPlaylist = $derived(
		resolvedMusic ? playlistStore.selectedPaths.includes(resolvedMusic.path) : false
	);

	function togglePlaylistSelection() {
		if (!resolvedMusic) return;
		const idx = playlistStore.selectedPaths.indexOf(resolvedMusic.path);
		if (idx >= 0) {
			playlistStore.selectedPaths = playlistStore.selectedPaths.filter(
				(p) => p !== resolvedMusic!.path
			);
		} else {
			playlistStore.selectedPaths = [...playlistStore.selectedPaths, resolvedMusic.path];
		}
	}

	// Use $effect with cleanup to cancel pending requests when component unmounts
	$effect(() => {
		// Only fetch image when visible
		const isVisible = getVisible();
		// Access dependencies synchronously to ensure tracking
		const currentMusic = music;
		const currentFolder = folder;

		if (!isVisible) return;

		if (currentFolder) {
			TauriLibraryAPI.getFolderInfo(currentFolder.path).then((info) => {
				if (!cancelled) {
					folderInfo = info;
				}
			});
		}

		let cancelled = false;
		const timeoutId = setTimeout(async () => {
			if (cancelled) return;
			const imagePromise = currentFolder
				? MetadataService.getFolderCoverArt(currentFolder.path)
				: currentMusic
					? MetadataService.getMusicCoverArt(currentMusic, CoverArtSize.MusicItem)
					: Promise.resolve(null);

			coverArt = imagePromise;

			// Track the blob URL for cleanup
			const url = await imagePromise;
			if (!cancelled && url) {
				// Revoke previous blob URL if exists
				if (currentBlobUrl) {
					URL.revokeObjectURL(currentBlobUrl);
				}
				currentBlobUrl = url;
			}
		}, COVER_ART_DEBOUNCE_DELAY);

		return () => {
			cancelled = true;
			clearTimeout(timeoutId);
			// Revoke blob URL on cleanup
			if (currentBlobUrl) {
				URL.revokeObjectURL(currentBlobUrl);
				currentBlobUrl = null;
			}
		};
	});

	const titleLabel = $derived.by(() => {
		if (folder) {
			return folderStore.currentFolder
				? folder.path.split(FolderService.PATH_SEPARATOR).pop()
				: folder.path;
		}
		return musicStore.listType === MusicListType.Folder ? music?.filename : music?.title;
	});

	const mediumLabel = $derived.by(() => {
		if (folder) return 'Folder';

		const album = music?.album ? `${music.album} ${MusicConfig.separatorAlbum} ` : '';
		const artist = music?.artist ?? MusicConfig.defaultArtist;
		return `${album}${artist}`;
	});

	const smallLabel = $derived.by(() => {
		if (folder) {
			if (!folderInfo) return '';
			const durationText = ProgressService.formatDuration(folderInfo.totalDuration);
			return `${folderInfo.trackCount} ${MusicConfig.separator} ${durationText}`;
		}

		const duration = ProgressService.formatDuration(music?.duration ?? 0);
		const resolution = [
			music?.bitsPerSample && `${music.bitsPerSample}-bit`,
			MetadataService.formatSampleRate(music?.sampleRate)
		].filter(Boolean);

		if (!resolution.length) return duration;

		const audioResolution = resolution.join(MusicConfig.separatorAudio);
		return `${audioResolution} ${MusicConfig.separator} ${duration}`;
	});

	async function addMusicAndPlay() {
		if (music) {
			const isFolderMode = musicStore.listType === MusicListType.Folder;
			const isPlaylistMode = musicStore.listType === MusicListType.Playlist;

			const filter = {
				search: filterStore.search,
				sortAsc: filterBarStore.sortAsc,
				albumName: filterStore.album?.name,
				folderPath:
					isFolderMode && folderStore.currentFolder ? folderStore.currentFolder.path : undefined,
				playlistPaths:
					isPlaylistMode && playlistStore.selectedPlaylist
						? playlistStore.selectedPlaylist.paths
						: undefined
			};
			await TauriLibraryAPI.collectionAddAndPlay({
				type: CollectionType.MusicIndex,
				index: musicIndex!,
				filter
			});
			musicStore.queueCount = 1;
			MusicPlayerService.play();
		} else if (folder) {
			await TauriLibraryAPI.collectionAddAndPlay({
				type: CollectionType.Folder,
				path: folder.path
			});
		}
	}

	async function addMusic() {
		if (music) {
			const isFolderMode = musicStore.listType === MusicListType.Folder;
			const isPlaylistMode = musicStore.listType === MusicListType.Playlist;

			const filter = {
				search: filterStore.search,
				sortAsc: filterBarStore.sortAsc,
				albumName: filterStore.album?.name,
				folderPath:
					isFolderMode && folderStore.currentFolder ? folderStore.currentFolder.path : undefined,
				playlistPaths:
					isPlaylistMode && playlistStore.selectedPlaylist
						? playlistStore.selectedPlaylist.paths
						: undefined
			};
			await TauriLibraryAPI.collectionAddToQueue({
				type: CollectionType.MusicIndex,
				index: musicIndex!,
				filter
			});
			musicStore.queueCount++;
			const title = music.title ?? music.filename ?? MusicConfig.defaultTitle;
			const artist = music.artist ?? MusicConfig.defaultArtist;
			ToastService.info(`Added music to queue: ${title} ${MusicConfig.separatorAlbum} ${artist}`);
		} else if (folder) {
			await TauriLibraryAPI.collectionAddToQueue({
				type: CollectionType.Folder,
				path: folder.path
			});
			const folderName = folder.path.split(FolderService.PATH_SEPARATOR).pop();
			ToastService.info(`Added folder to queue: ${folderName}`);
		}
	}

	async function selectFolder() {
		if (folder) folderStore.currentFolder = folder;
	}

	const isVisible = $derived.by(() => {
		const search = filterStore.search.toLowerCase();

		if (folder) {
			return folder.path.toLowerCase().includes(search);
		}

		if (!music) return false;

		const album = filterStore.album;
		const hasSearch = search.length > 0;
		const matchesSearch =
			!hasSearch ||
			[music.album, music.title, music.artist, music.albumArtist].some((v) =>
				v?.toLowerCase().includes(search)
			);

		const hasAlbum = !!album;
		const matchesAlbum = !hasAlbum || album.name === music.album;

		return matchesSearch && matchesAlbum;
	});

	return {
		get isVisible() {
			return isVisible;
		},
		get coverArt() {
			return coverArt;
		},
		get titleLabel() {
			return titleLabel;
		},
		get mediumLabel() {
			return mediumLabel;
		},
		get smallLabel() {
			return smallLabel;
		},
		get resolvedMusic() {
			return resolvedMusic;
		},
		get isSelectedForPlaylist() {
			return isSelectedForPlaylist;
		},
		togglePlaylistSelection,
		addMusicAndPlay,
		addMusic,
		selectFolder
	};
}
