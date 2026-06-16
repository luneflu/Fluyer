import { PageRoutes } from '$lib/constants/PageRoutes';
import MetadataService from '$lib/services/MetadataService.svelte';
import musicStore from '$lib/stores/music.svelte';
import settingStore from '$lib/stores/setting.svelte';
import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';
import ProgressService from '$lib/services/ProgressService.svelte';
import PageService from '$lib/services/PageService.svelte';
import playerBarStore from '$lib/stores/playerBar.svelte';
import QueueService from '$lib/services/QueueService.svelte';
import { CoverArtSize } from '$lib/services/CoverArtService.svelte';

let element = $state<HTMLDivElement>();
let coverArt = $state<Promise<string | null> | null>(null);
let currentBlobUrl: string | null = null;
let currentMusicPath: string | null = null;

let progressPercentage = $state(0);
let volumePercentage = $state(0);

const isPlaying = $derived(musicStore.isPlaying);

const gridRight = $derived.by(() => {
	if (settingStore.ui.showRepeatButton && settingStore.ui.showShuffleButton)
		return 'grid-cols-[repeat(5,auto)]';
	if (settingStore.ui.showRepeatButton && settingStore.ui.showShuffleButton)
		return 'grid-cols-[repeat(4,auto)]';
	return 'grid-cols-[repeat(3,auto)]';
});

function handleButtonPlayPause() {
	if (musicStore.isPlaying) {
		musicStore.isPlaying = false;
		MusicPlayerService.pause();
	} else {
		MusicPlayerService.play();
	}
}

function handleButtonPrevious() {
	MusicPlayerService.previous();
}

function handleButtonNext() {
	MusicPlayerService.next();
}

async function handleButtonShuffle() {
	await MusicPlayerService.pause();

	await QueueService.shuffleQueue();

	await MusicPlayerService.play();
	ProgressService.start();
}

function redirectToPlay() {
	PageService.goTo(PageRoutes.PLAY);
}

function handleVolumeButton() {
	musicStore.volume = musicStore.volume > 0 ? 0 : 1;
}

function handleProgressClick(percentage: number) {
	MusicPlayerService.seekByPercentage(percentage);
}

function handleVolumeProgressClick(percentage: number) {
	musicStore.volume = percentage / 100;
}

function updatePlayerBarHeight() {
	if (element) {
		playerBarStore.height = element.offsetHeight;
	}
}

export function usePlayerBar() {
	// Fetch album image with blob URL cleanup
	$effect(() => {
		let cancelled = false;

		(async () => {
			const newMusicPath = musicStore.currentMusic?.path;
			currentMusicPath = newMusicPath ?? null;

			const imagePromise = MetadataService.getMusicCoverArt(
				musicStore.currentMusic,
				CoverArtSize.PlayerBar
			);
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

	$effect(() => {
		progressPercentage = musicStore.progressPercentage;
	});

	$effect(() => {
		volumePercentage = musicStore.volumePercentage;
	});

	return {
		get element() {
			return element;
		},
		set element(value) {
			element = value;
			updatePlayerBarHeight();
		},
		get coverArt() {
			return coverArt;
		},
		get isPlaying() {
			return isPlaying;
		},
		get progressPercentage() {
			return progressPercentage;
		},
		get volumePercentage() {
			return volumePercentage;
		},
		get gridRight() {
			return gridRight;
		},

		handleButtonPlayPause,
		handleButtonPrevious,
		handleButtonNext,
		handleButtonShuffle,
		redirectToPlay,
		handleVolumeButton,
		handleProgressClick,
		handleVolumeProgressClick,
		updatePlayerBarHeight
	};
}
