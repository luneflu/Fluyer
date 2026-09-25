import { PageRoutes } from '$lib/constants/PageRoutes';
import { page } from '$app/state';
import TauriMobileAPI from '$lib/tauri/TauriMobileAPI';
import { isAndroid } from '$lib/platform';
import mobileStore from '$lib/stores/mobile.svelte';

const MobileService = {
	initialize: async () => {
		if (!isAndroid()) return;
		MobileService.listenPageEvents();
		TauriMobileAPI.getNavigationBarHeight().then((height) => {
			mobileStore.navigationBarHeight = height;
		});
		TauriMobileAPI.getStatusBarHeight().then((height) => {
			mobileStore.statusBarHeight = height;
		});
	},
	listenPageEvents: () => {
		$effect(() => {
			const visible = page.url.pathname !== PageRoutes.PLAY;
			TauriMobileAPI.setNavigationBarVisibility(visible);
		});
	}
};

export default MobileService;
