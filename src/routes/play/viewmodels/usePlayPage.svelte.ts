import { isDesktop, isMacos, isWindows } from '$lib/platform';
import musicStore from '$lib/stores/music.svelte';
import ProgressService from '$lib/services/ProgressService.svelte';
import MetadataService from '$lib/services/MetadataService.svelte';
import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';
import PageService from '$lib/services/PageService.svelte';
import QueueService from '$lib/services/QueueService.svelte';
import LyricService, { type MusicLyric } from '$lib/services/LyricService.svelte';
import { getCurrentWindow } from '@tauri-apps/api/window';
import TauriBackgroundAPI from '$lib/tauri/TauriBackgroundAPI';

let lyricContainerElement: HTMLDivElement;

const music = $derived(musicStore.currentMusic);
const progressPercentage = $derived.by(() => musicStore.progressPercentage);

let progressDurationText = $state('');
let progressDurationNegativeText = $state('');
let updateProgressText = $state(true);

let coverArt = $state<Promise<string | null> | null>(null);
let currentBlobUrl: string | null = null;

let lyrics = $state<MusicLyric[]>([]);
let selectedLyricIndex = $state(0);

let volumePercentage = $state(musicStore.volume);
let hideBackButton = $state(false);
let isIdle = $state(false);
let timer: NodeJS.Timeout;

function resetIdleTimer() {
	isIdle = false;
	clearTimeout(timer);
	timer = setTimeout(() => {
		isIdle = true;
	}, 3000);
}

function handleBackWithDelay() {
	hideBackButton = true;
	setTimeout(
		() => {
			handleButtonBack();
		},
		isMacos() ? 300 : 0
	);
}

function onKeyDown(
	e: KeyboardEvent & {
		currentTarget: EventTarget & Document;
	}
) {
	if (e.key === 'Escape') handleBackWithDelay();
	resetIdleTimer();
}

function scrollToSelectedLyric() {
	// Wait for the next frame so the selected lyric's size has updated
	requestAnimationFrame(() => {
		document.getElementById('selected-lyric')?.scrollIntoView({
			block: window.innerWidth > 768 ? 'center' : 'start',
			behavior: 'smooth'
		});
	});
}

function handleButtonPlayPause() {
	if (musicStore.isPlaying) {
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

async function handleButtonBack() {
	if (isDesktop()) {
		await getCurrentWindow().setFullscreen(false);
		if (isWindows()) {
			await TauriBackgroundAPI.restoreBackground();
		}
	}
	PageService.back();
}

async function handleButtonShuffle() {
	await MusicPlayerService.pause();
	await QueueService.shuffleQueue();
	await MusicPlayerService.play();
}

async function resetLyrics() {
	selectedLyricIndex = 0;
	lyrics = [];

	if (!musicStore.currentMusic) return;
	const resLyrics = await LyricService.get(music);
	if (resLyrics == null) {
		lyrics = [];
		return;
	}
	lyrics = resLyrics;
}

function resetSelectedLyricIndex() {
	if (lyrics.length < 1) return;

	const duration = musicStore.progressDuration / 1000;
	if (duration < lyrics[0].duration) {
		return;
	}
	for (let i = 0; i < lyrics.length; i++) {
		if (duration < lyrics[i].duration) {
			selectedLyricIndex = i - 1;
			return;
		}
	}
	selectedLyricIndex = lyrics.length - 1;
}

function refreshProgressText() {
	if (!updateProgressText) return;
	progressDurationText = ProgressService.formatDuration(musicStore.progressDuration);
	progressDurationNegativeText =
		'-' +
		ProgressService.formatDuration(
			(musicStore.currentMusic?.duration ?? 0) - musicStore.progressDuration
		);
}

function handleProgressClick(percentage: number) {
	MusicPlayerService.seekByPercentage(percentage);
}

function handleProgressEnter() {
	updateProgressText = false;
}

function handleProgressMove(percentage: number) {
	updateProgressText = false;
	progressDurationText = ProgressService.formatDuration(
		(musicStore.currentMusic?.duration ?? 0) * (percentage / 100)
	);
	progressDurationNegativeText =
		'-' +
		ProgressService.formatDuration(
			(musicStore.currentMusic?.duration ?? 0) * ((100 - percentage) / 100)
		);
}

function handleProgressLeave() {
	updateProgressText = true;
	refreshProgressText();
}

function handleVolumeProgressClick(percentage: number) {
	musicStore.volume = percentage / 100;
}

export function usePlayPage() {
	hideBackButton = false;
	$effect(() => {
		musicStore.progressPercentage;
		refreshProgressText();
		resetSelectedLyricIndex();
	});

	$effect(() => {
		musicStore.currentIndex;
		let cancelled = false;

		(async () => {
			const imagePromise = MetadataService.getMusicCoverArt(musicStore.currentMusic);
			coverArt = imagePromise;

			const url = await imagePromise;
			if (!cancelled && url && url.startsWith('blob:')) {
				if (currentBlobUrl) {
					URL.revokeObjectURL(currentBlobUrl);
				}
				currentBlobUrl = url;
			}
		})();

		resetLyrics();

		return () => {
			cancelled = true;
			if (currentBlobUrl) {
				URL.revokeObjectURL(currentBlobUrl);
				currentBlobUrl = null;
			}
		};
	});

	$effect(() => {
		volumePercentage = musicStore.volumePercentage;
	});

	$effect(() => {
		resetIdleTimer();
		return () => clearTimeout(timer);
	});

	$effect(() => {
		selectedLyricIndex;
		if (typeof document !== 'undefined') {
			scrollToSelectedLyric();
		}
	});

	return {
		get music() {
			return music;
		},
		get progressPercentage() {
			return progressPercentage;
		},
		get progressDurationText() {
			return progressDurationText;
		},
		get progressDurationNegativeText() {
			return progressDurationNegativeText;
		},
		get coverArt() {
			return coverArt;
		},
		get lyrics() {
			return lyrics;
		},
		get selectedLyricIndex() {
			return selectedLyricIndex;
		},
		get volumePercentage() {
			return volumePercentage;
		},
		set updateProgressText(val: boolean) {
			updateProgressText = val;
		},
		get hideBackButton() {
			return hideBackButton;
		},
		get isIdle() {
			return isIdle;
		},
		get lyricContainerElement() {
			return lyricContainerElement;
		},
		set lyricContainerElement(value: HTMLDivElement) {
			lyricContainerElement = value;
		},

		handleButtonPlayPause,
		handleButtonPrevious,
		handleButtonNext,
		handleButtonBack,
		handleButtonShuffle,
		handleProgressClick,
		handleProgressEnter,
		handleProgressMove,
		handleProgressLeave,
		handleVolumeProgressClick,
		resetIdleTimer,
		handleBackWithDelay,
		onKeyDown
	};
}
