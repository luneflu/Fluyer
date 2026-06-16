import musicStore from '$lib/stores/music.svelte';
import { MusicConfig } from '$lib/constants/MusicConfig';
import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';

const ProgressService = {
	initialize: () => {},
	start: () => {
		if (musicStore.progressIntervalId !== null) {
			console.warn("Can't start progress because it is already running.");
			return;
		}

		console.log(`Starting progress with duration: ${musicStore.currentMusic?.duration}`);

		const updateInterval = (musicStore.currentMusic!.duration / MusicConfig.max) * MusicConfig.step;

		musicStore.progressIntervalId = setInterval(() => {
			musicStore.progressValue += MusicConfig.step;

			if (musicStore.progressValue >= MusicConfig.max) {
				console.log('Progress value ended. Stopping...');
				MusicPlayerService.pause();
			}
		}, updateInterval);
	},
	stop: () => {
		if (musicStore.progressIntervalId === null) {
			console.warn("Can't stop progress because it is not running.");
			return;
		}
		console.log('Stopping progress...');
		clearInterval(musicStore.progressIntervalId);
		musicStore.progressIntervalId = null;
	},
	reset: () => {
		ProgressService.stop();
		musicStore.progressValue = 0;
	},
	formatDuration: (duration: number, negative?: boolean) => {
		duration = duration / 1000;
		const totalSeconds = Math.max(0, duration);
		let minutes = Math.floor(totalSeconds / 60);
		let secs = Math.round(totalSeconds % 60);

		if (secs === 60) {
			minutes += 1;
			secs = 0;
		}

		return `${negative ? '-' : ''}${minutes}:${secs.toString().padStart(2, '0')}`;
	}
};

export default ProgressService;
