import musicStore from '$lib/stores/music.svelte';
import ProgressService from '$lib/services/ProgressService.svelte';
import TauriMusicAPI from '$lib/tauri/TauriMusicAPI';
import TauriLibraryAPI from '$lib/tauri/TauriLibraryAPI';
import QueueService from '$lib/services/QueueService.svelte';
import { RepeatMode } from '$lib/features/music/types';
import PersistentStoreService from '$lib/services/PersistentStoreService.svelte';
import { MusicConfig } from '$lib/constants/MusicConfig';
import { isDesktop } from '$lib/platform';

const MusicPlayerService = {
	initialize: async () => {
		MusicPlayerService.listenSyncEvents();
		MusicPlayerService.listenVolumeEvents();

		if(isDesktop()){
			const discordRpcEnabled = await PersistentStoreService.discordRpcEnabled.get();
			await TauriMusicAPI.setDiscordRpcEnabled(discordRpcEnabled ?? false);
		}
	},
	play: async () => {
		if (musicStore.queueCount === 0) {
			console.warn("Can't play music playback because the queue is empty.");
			return;
		}

		if (musicStore.isPlaying) {
			console.warn('Playing music playback called but music is already played.');
		}

		console.log('Starting music playback...');

		musicStore.isPlaying = true;
		await TauriMusicAPI.play();
		ProgressService.start();
	},
	pause: async () => {
		if (!musicStore.isPlaying) {
			console.warn('Pausing music playback called but music is already paused.');
		}

		console.log('Pausing music playback...');
		musicStore.isPlaying = false;
		await TauriMusicAPI.pause();
		ProgressService.stop();
	},
	next: async () => {
		return TauriMusicAPI.next();
	},
	previous: async () => {
		if (musicStore.queueCount === 0) return;
		const prevIndex =
			musicStore.currentIndex > 0 ? musicStore.currentIndex - 1 : musicStore.queueCount - 1;
		return QueueService.goTo(prevIndex);
	},
	seekByPercentage: async (percentage: number) => {
		const clamped = Math.min(100, Math.max(0, percentage));
		const position = (musicStore.currentMusic?.duration ?? 0) * (clamped / 100);

		await TauriMusicAPI.setPosition(position);
		await TauriMusicAPI.requestSync();
	},
	toggleRepeatMode: async () => {
		let nextRepeatMode = RepeatMode.None;
		const currentMode = musicStore.repeatMode;

		switch (currentMode) {
			case RepeatMode.None:
				nextRepeatMode = RepeatMode.All;
				break;
			case RepeatMode.All:
				nextRepeatMode = RepeatMode.One;
				break;
			case RepeatMode.One:
				nextRepeatMode = RepeatMode.None;
				break;
		}

		// Optimistic update
		musicStore.repeatMode = nextRepeatMode;
		await TauriMusicAPI.setRepeatMode(nextRepeatMode);
	},

	listenSyncEvents: () => {
		return TauriMusicAPI.listenSync(async (e) => {
			console.log('Received music player sync event:', e.payload);

			if (e.payload.index > -1) {
				const musicData = await TauriLibraryAPI.getQueueByIndex(e.payload.index);

				if (!musicStore.currentMusic || musicStore.currentMusic.path !== musicData?.path) {
					if (musicData) {
						musicStore.currentIndex = e.payload.index;
						musicStore.currentMusic = musicData;
					}
				} else if (musicStore.currentIndex !== e.payload.index) {
					musicStore.currentIndex = e.payload.index;
				}

				if (musicStore.currentMusic) {
					musicStore.progressValue =
						(e.payload.currentPosition / musicStore.currentMusic.duration) * MusicConfig.max;
				}
			} else {
				musicStore.currentIndex = -1;
				musicStore.currentMusic = undefined;
				ProgressService.reset();
			}

			musicStore.isPlaying = e.payload.isPlaying;
			musicStore.repeatMode = e.payload.repeatMode;
			musicStore.isShuffled = e.payload.isShuffled;

			if (e.payload.isPlaying) {
				ProgressService.stop();
				ProgressService.start();
			} else ProgressService.stop();

			QueueService.refreshCount();
		});
	},
	listenVolumeEvents: () => {
		$effect(() => {
			(async () => {
				await PersistentStoreService.volume.set(musicStore.volume);
				await TauriMusicAPI.setVolume(musicStore.volume);
			})();
		});
	}
};

export default MusicPlayerService;
