<script lang="ts">
	import SettingLabel from '$lib/features/settings/SettingLabel.svelte';
	import SettingInput from '$lib/features/settings/SettingInput.svelte';
	import { isDesktop } from '$lib/platform';
	import settingStore from '$lib/stores/setting.svelte.js';
	import PersistentStoreService from '$lib/services/PersistentStoreService.svelte.js';
	import TauriMusicAPI from '$lib/tauri/TauriMusicAPI';
	import ToastService from '$lib/services/ToastService.svelte.js';

	function onDiscordRpcChange(
		e: Event & {
			currentTarget: EventTarget & HTMLInputElement;
		}
	) {
		settingStore.discordRpcEnabled = e.currentTarget.checked;
		PersistentStoreService.discordRpcEnabled.set(e.currentTarget.checked);
		TauriMusicAPI.setDiscordRpcEnabled(e.currentTarget.checked);
		if (e.currentTarget.checked) TauriMusicAPI.requestSync();
		ToastService.info(
			`Discord Rich Presence is ${e.currentTarget.checked ? 'enabled' : 'disabled'}`
		);
	}
</script>

{#if isDesktop()}
	<SettingLabel
		title="Discord"
		description="Show what you're listening to as your Discord status."
	/>

	<SettingInput>
		<label class="grid cursor-pointer grid-cols-[min-content_auto] items-center gap-3 px-3 py-2">
			<input
				type="checkbox"
				class="h-4 w-4"
				checked={settingStore.discordRpcEnabled}
				onchange={onDiscordRpcChange}
			/>
			<div>Discord Rich Presence</div>
		</label>
	</SettingInput>
{/if}
