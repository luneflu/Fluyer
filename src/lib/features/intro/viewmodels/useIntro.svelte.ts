import { isAndroid } from '$lib/platform';
import TauriIntroAPI from '$lib/tauri/TauriIntroAPI';
import PersistentStoreService from '$lib/services/PersistentStoreService.svelte';
import LibraryService from '$lib/services/LibraryService.svelte';

export function useIntro() {
	let animatedClasses = $state('anim-fade-in');

	async function requestAction() {
		if (isAndroid()) {
			const result = await TauriIntroAPI.requestReadAudioPermission();
			if (!result) return;
		}
		await requestDirectoryPath();
	}

	async function requestDirectoryPath() {
		const path = await TauriIntroAPI.requestDirectoryPath();
		if (isAndroid()) await PersistentStoreService.musicPath.set([path]);

		animatedClasses = 'anim-fade-out';
	}

	function onAnimationEnd(currentClass: string) {
		if (currentClass === 'anim-fade-in') return;
		LibraryService.initialize();
	}

	return {
		get animatedClasses() {
			return animatedClasses;
		},
		requestAction,
		onAnimationEnd
	};
}
