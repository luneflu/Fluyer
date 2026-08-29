<script lang="ts">
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import SettingMusicPaths from '$lib/features/settings/music_paths/SettingMusicPaths.svelte';
	import { isMobile } from '$lib/platform';
	import SettingAnimatedBackground from '$lib/features/settings/animated_background/SettingAnimatedBackground.svelte';
	import SettingUserInteface from '$lib/features/settings/user_interface/SettingUserInterface.svelte';
	import SettingDeveloper from '$lib/features/settings/developer/SettingDeveloper.svelte';
	import SettingIconTheme from '$lib/features/settings/icon_theme/SettingIconTheme.svelte';
	import SettingDiscordRpc from '$lib/features/settings/discord_rpc/SettingDiscordRpc.svelte';
	import View from '$lib/ui/components/View.svelte';
	import Button from '$lib/ui/components/Button.svelte';
	import mobileStore from '$lib/stores/mobile.svelte';
	import PageService from '$lib/services/PageService.svelte';
	import ToastService from '$lib/services/ToastService.svelte';
	import { onMount } from 'svelte';

	let disableBorder = $state(false);
	let isSmallScreen = $state(false);
	let hideTitle = $derived(isSmallScreen && ToastService.toasts.length > 0);

	function onResize() {
		disableBorder = window.innerWidth < 768;
		isSmallScreen = window.innerWidth < 768;
	}
	onMount(onResize);
</script>

<svelte:window onresize={onResize} />
<div class="h-full w-full md:px-3 md:pb-4 md:pt-10">
	<View
		class="h-full w-full rounded-lg !bg-transparent {disableBorder && '!border-0 !shadow-none'}"
		glassShineSize="xs"
	>
		<div
			class="grid h-full w-full grid-rows-[min-content_auto_min-content]"
			style="padding-top: {isMobile() ? mobileStore.statusBarHeight : 20}px;
                padding-bottom: {mobileStore.navigationBarHeight > 0
				? mobileStore.navigationBarHeight
				: 24}px;"
		>
			<p
				class="anim mx-5 mb-4 text-2xl font-semibold {hideTitle ? 'anim-fade-out' : 'anim-fade-in'}"
				style="animation-duration: 500ms; {hideTitle ? 'opacity: 0;' : 'opacity: 1;'}"
			>
				Settings
			</p>
			<div class="scrollbar-hidden mb-3 w-full overflow-auto">
				<!-- <SettingMusicPlayerConfiguration /> -->
				<SettingMusicPaths />
				<SettingAnimatedBackground />
				<SettingIconTheme />
				<SettingDiscordRpc />
				<SettingUserInteface />
				<SettingDeveloper />
			</div>
			<Button
				class="mx-5 grid w-fit grid-cols-[min-content_auto] items-center gap-2
                rounded bg-transparent px-3 py-2"
				onclick={() => PageService.back()}
			>
				<div class="w-4"><Icon type={IconType.Back} /></div>
				<div>Back</div>
			</Button>
		</div>
	</View>
</div>
