import TauriMusicAPI from '$lib/tauri/TauriMusicAPI';
import musicStore from '$lib/stores/music.svelte';
import TauriQueueAPI from '$lib/tauri/TauriQueueAPI';
import TauriLibraryAPI from '$lib/tauri/TauriLibraryAPI';
import { invoke } from '@tauri-apps/api/core';
import { TauriCommands } from '$lib/constants/TauriCommands';
import type { MusicData } from '$lib/features/music/types';

class PlaylistMoveQueue {
	private queue: Array<() => Promise<void>> = [];
	private isProcessing = false;

	async add(operation: () => Promise<void>): Promise<void> {
		return new Promise((resolve, reject) => {
			this.queue.push(async () => {
				try {
					await operation();
					resolve();
				} catch (error) {
					reject(error);
				}
			});

			if (!this.isProcessing) {
				this.processQueue();
			}
		});
	}

	private async processQueue(): Promise<void> {
		if (this.isProcessing || this.queue.length === 0) return;

		this.isProcessing = true;

		while (this.queue.length > 0) {
			const operation = this.queue.shift();
			if (operation) {
				try {
					await operation();
				} catch (error) {
					console.error('Queue operation failed:', error);
				}
			}
		}

		this.isProcessing = false;
	}
}

const playlistMoveQueue = new PlaylistMoveQueue();

/**
 * Queue service – Rust owns the playlist; we only send commands and update
 * `musicStore.queueCount` (and `currentIndex`) based on Tauri sync events.
 */
const QueueService = {
	remove: async (index: number) => {
		await TauriQueueAPI.remove(index);
		// currentIndex adjustment is handled by the music_player_sync event
	},

	goTo: (index: number) => {
		return TauriQueueAPI.goTo(index);
	},

	moveTo: (from: number, to: number) => {
		if (from === to) return Promise.resolve();

		return playlistMoveQueue.add(async () => {
			await TauriQueueAPI.moveTo(from, to);
		});
	},

	shuffleQueue: async () => {
		await invoke<void>(TauriCommands.MUSIC_QUEUE_SHUFFLE);
	},

	clear: async () => {
		await TauriMusicAPI.clear();
		musicStore.queueCount = 0;
		musicStore.currentIndex = -1;
		musicStore.currentMusic = undefined;
	},

	refreshCount: async () => {
		musicStore.queueCount = await TauriLibraryAPI.getQueueCount();
	}
};

export default QueueService;
