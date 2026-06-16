import filterStore from '$lib/stores/filter.svelte';
import filterBarStore from '$lib/stores/filterBar.svelte';
import MetadataService from '$lib/services/MetadataService.svelte';
import musicStore from '$lib/stores/music.svelte';
import { type AlbumData, type MusicData, MusicListType } from '$lib/features/music/types';
import ProgressService from '$lib/services/ProgressService.svelte';
import { COVER_ART_DEBOUNCE_DELAY, CoverArtSize } from '$lib/services/CoverArtService.svelte';
import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';
import TauriLibraryAPI, { CollectionType } from '$lib/tauri/TauriLibraryAPI';

export function useAlbumItem(
	getAlbumIndex: () => number,
	getIndex: () => number,
	getVisible: () => boolean = () => true
) {
	const albumIndex = $derived(getAlbumIndex());
	const index = $derived(getIndex());

	// Fetched from Rust when visible
	let music = $state<MusicData | null>(null);
	let coverArt = $state<Promise<string | null> | null>(null);
	let currentBlobUrl: string | null = null;

	const isValidFilterAlbum = $derived(
		filterStore.album && music?.album && filterStore.album.name === music.album
	);

	$effect(() => {
		const isVisible = getVisible();
		if (!isVisible) return;

		TauriLibraryAPI.getAlbumFirstByIndex(
			albumIndex,
			filterStore.search,
			filterBarStore.sortAsc
		).then((m) => {
			if (m) music = m;
		});
	});

	$effect(() => {
		const m = music;
		const isVisible = getVisible();
		if (!isVisible || !m) return;

		let cancelled = false;
		const timeoutId = setTimeout(async () => {
			if (cancelled) return;
			const imagePromise = MetadataService.getMusicCoverArt(m, CoverArtSize.AlbumItem);
			coverArt = imagePromise;

			const url = await imagePromise;
			if (!cancelled && url) {
				if (currentBlobUrl) {
					URL.revokeObjectURL(currentBlobUrl);
				}
				currentBlobUrl = url;
			}
		}, COVER_ART_DEBOUNCE_DELAY);

		return () => {
			cancelled = true;
			clearTimeout(timeoutId);
			if (currentBlobUrl) {
				URL.revokeObjectURL(currentBlobUrl);
				currentBlobUrl = null;
			}
		};
	});

	async function setFilterAlbum() {
		if (!music) return;
		const tracks = await TauriLibraryAPI.getAlbumByIndex(
			albumIndex,
			filterStore.search,
			filterBarStore.sortAsc
		);
		if (!tracks) return;

		const isAlbumType = musicStore.listType === MusicListType.Album;
		musicStore.listType = MusicListType.All;
		filterStore.album = {
			name: music.album,
			artist: music.albumArtist ?? music.artist,
			year: MetadataService.getYearFromDate(music.date),
			duration: ProgressService.formatDuration(
				tracks.map((m) => m.duration).reduce((a, b) => a + b, 0)
			),
			tracks
		} as AlbumData;
		if (isAlbumType) setTimeout(() => (musicStore.albumsUi.scrollIndex = index), 500);
	}

	async function playAlbum() {
		if (!music) return;
		await TauriLibraryAPI.collectionAddAndPlay({
			type: CollectionType.AlbumIndex,
			index: albumIndex,
			search: filterStore.search,
			sortAsc: filterBarStore.sortAsc
		});
		MusicPlayerService.play();
	}

	return {
		get isValidFilterAlbum() {
			return isValidFilterAlbum;
		},
		get coverArt() {
			return coverArt;
		},
		get music() {
			return music;
		},
		setFilterAlbum,
		playAlbum
	};
}
