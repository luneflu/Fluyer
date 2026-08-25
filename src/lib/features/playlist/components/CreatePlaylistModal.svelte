<script lang="ts">
	import playlistStore from '$lib/stores/playlist.svelte';
	import { Modal } from '$lib/constants/Modal';
	import type { PlaylistData } from '$lib/features/music/types';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import Button from '$lib/ui/components/Button.svelte';
	import View from '$lib/ui/components/View.svelte';
	import Input from '$lib/ui/components/Input.svelte';
	import modalStore from '$lib/stores/modal.svelte';
	import PlaylistService from '$lib/services/PlaylistService.svelte';

	let title = $state('');
	let artist = $state('');
	let selectedMusicPath = $state('');
	let uploadedImagePath = $state('');
	let isSubmitting = $state(false);

	async function handleImageUpload() {
		uploadedImagePath = await PlaylistService.requestUploadImage();
	}

	async function handleSubmit() {
		if (!title.trim()) return;
		isSubmitting = true;

		const playlist: PlaylistData = {
			name: title.trim(),
			title: title.trim(),
			artist: artist.trim() || undefined,
			image: uploadedImagePath || undefined,
			paths: playlistStore.selectedPaths
		};

		await PlaylistService.create(playlist);
		await PlaylistService.loadPlaylist();

		close();
		isSubmitting = false;
	}

	function close() {
		title = '';
		artist = '';
		selectedMusicPath = '';
		uploadedImagePath = '';

		PlaylistService.cancelCreation();
	}
</script>

{#if modalStore.show && modalStore.type === Modal.CreatePlaylist}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center"
		onclick={(e) => {
			if (e.target === e.currentTarget) close();
		}}
	>
		<View
			class="anim anim-fade-in mx-3 w-full rounded
			p-10 md:w-[30rem]"
		>
			<h2 class="text-lg font-semibold text-white">Create Playlist</h2>

			<div class="mt-4 space-y-4">
				<!-- Title -->
				<div>
					<div class="mb-1 block text-sm font-medium">Title</div>
					<Input class="rounded" bind:value={title} placeholder="Playlist name..." />
				</div>

				<!-- Artist -->
				<div>
					<div class="mb-1 block text-sm font-medium">Artist</div>
					<Input class="rounded" bind:value={artist} placeholder="Artist name..." />
				</div>

				<!-- Image -->
				<div>
					<div class="block text-sm font-medium text-white">Image</div>
					<div class="mb-2 text-xs text-white/70">Leave empty to use default image</div>
					<Button
						class="grid w-fit grid-cols-[min-content_auto] items-center gap-x-2 rounded px-2 py-1 text-sm"
						onclick={handleImageUpload}
					>
						<div class="h-6 w-6">
							<Icon type={IconType.Image} />
						</div>
						<div>Upload Image</div>
					</Button>
					{#if uploadedImagePath}
						<div class="mt-2 text-xs text-white/70">Image uploaded successfully</div>
					{/if}
				</div>

				<!-- Actions -->
				<div class="flex justify-end gap-2 pt-2">
					<Button class="rounded px-3 py-1" onclick={close}>Cancel</Button>
					<Button class="rounded px-3 py-1" onclick={handleSubmit}>
						{isSubmitting ? 'Creating...' : 'Create'}
					</Button>
				</div>
			</div>
		</View>
	</div>
{/if}
