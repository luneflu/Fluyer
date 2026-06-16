<script lang="ts">
	import { PageRoutes } from '$lib/constants/PageRoutes';
	import { IconType } from '$lib/ui/icon/types';
	import Sidebar from '$lib/features/sidebar/components/Sidebar.svelte';
	import MenuButton from '$lib/features/menu/components/MenuButton.svelte';
	import { SidebarType } from '$lib/features/sidebar/types';
	import PageService from '$lib/services/PageService.svelte';
	import MenuVolume from './MenuVolume.svelte';
	import { isDesktop, isWindows } from '$lib/platform';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import TauriBackgroundAPI from '$lib/tauri/TauriBackgroundAPI';

	async function gotoPlayPage() {
		if (isDesktop()) {
			await getCurrentWindow().setFullscreen(true);
			if (isWindows()) {
				await TauriBackgroundAPI.restoreBackground();
			}
		}
		PageService.goTo(PageRoutes.PLAY);
	}
</script>

<Sidebar type={SidebarType.Left}>
	<p class="px-3 py-2 text-[1.2rem] font-semibold md:text-[1.5rem]">Menu</p>

	<MenuButton label="Play Screen" icon={IconType.Fullscreen} onclick={gotoPlayPage} />
	<!--{#if isDesktop() && !$settingBitPerfectMode}-->
	<!--    <MenuButton label="Equalizer" icon={IconType.Equalizer}-->
	<!--              onclick={() => UIController.toggleEqualizer(true)}/>-->
	<!--{/if}-->
	<!-- <MenuButton
		label="Visualizer"
		icon={IconType.Visualizer}
		onclick={() => PageService.goTo(PageRoutes.VISUALIZER)}
	/> -->
	<MenuButton
		label="Settings"
		icon={IconType.Settings}
		onclick={() => PageService.goTo(PageRoutes.SETTINGS)}
	/>
	<MenuVolume />
</Sidebar>
