import type { MusicData } from '$lib/features/music/types';
import musicStore from '$lib/stores/music.svelte';
import MetadataService from '$lib/services/MetadataService.svelte';
import QueueService from '$lib/services/QueueService.svelte';
import { CoverArtSize } from '$lib/services/CoverArtService.svelte';
import TauriLibraryAPI from '$lib/tauri/TauriLibraryAPI';

export function useMusicQueueItem(getIndex: () => number, getVisible: () => boolean = () => true) {
	let coverArt = $state<Promise<string | null> | null>(null);
	let currentBlobUrl: string | null = null;
	let music = $state<MusicData | null>(null);

	const index = $derived(getIndex());
	const visible = $derived(getVisible());

	const isPlaying = $derived(musicStore.currentIndex === index);
	const isPrevious = $derived(index < musicStore.currentIndex);

	$effect(() => {
		const isVisible = visible;
		const revision = musicStore.queueRevision; // track queue revision
		if (!isVisible) return;

		TauriLibraryAPI.getQueueByIndex(index).then((m) => {
			music = m;
		});
	});

	$effect(() => {
		const currentMusic = music;
		const isVisible = visible;

		if (!isVisible || !currentMusic) return;

		let cancelled = false;

		(async () => {
			const imagePromise = MetadataService.getMusicCoverArt(currentMusic, CoverArtSize.QueueItem);
			coverArt = imagePromise;

			const url = await imagePromise;
			if (!cancelled && url && url.startsWith('blob:')) {
				if (currentBlobUrl) {
					URL.revokeObjectURL(currentBlobUrl);
				}
				currentBlobUrl = url;
			}
		})();

		return () => {
			cancelled = true;
			if (currentBlobUrl) {
				URL.revokeObjectURL(currentBlobUrl);
				currentBlobUrl = null;
			}
		};
	});

	function removePlaylist() {
		QueueService.remove(index);
	}

	function goToPlaylist() {
		QueueService.goTo(index);
	}

	return {
		get isPlaying() {
			return isPlaying;
		},
		get isPrevious() {
			return isPrevious;
		},
		get coverArt() {
			return coverArt;
		},
		get music() {
			return music;
		},
		removePlaylist,
		goToPlaylist
	};
}
