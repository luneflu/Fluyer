<script lang="ts">
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import { isLinux, isMacos, isMobile, isWindows } from '$lib/platform';
	import Button from '$lib/ui/components/Button.svelte';
	import Input from '$lib/ui/components/Input.svelte';
	import filterStore from '$lib/stores/filter.svelte';
	import filterBarStore from '$lib/stores/filterBar.svelte';
	import mobileStore from '$lib/stores/mobile.svelte';
	import musicStore from '$lib/stores/music.svelte';
	import playlistStore from '$lib/stores/playlist.svelte';
	import { useFilterBar } from '../viewmodels/useFilterBar.svelte';
	import modalStore from '$lib/stores/modal.svelte';
	import ConfirmCancelButtons from './ConfirmCancelButtons.svelte';
	import ListModeSelector from './ListModeSelector.svelte';
	import AddPlaylistButton from './AddPlaylistButton.svelte';
	import appStore from '$lib/stores/app.svelte';

	const vm = useFilterBar();
</script>

<svelte:window onresize={vm.updateSize} />
<div
	class="pointer-events-none fixed left-0 top-0 z-20 grid w-full gap-y-2 pb-3
        {isMacos() ? 'sm:justify-end' : ''}
        {isMacos() ? 'right-0' : 'left-0'}
        {modalStore.show ? 'opacity-10 blur-sm' : ''} anim anim-slide-in-down
        transition-opacity duration-300"
	style="margin-top: {isMobile() ? mobileStore.statusBarHeight : 8}px;
        grid-template-columns: {vm.state.gridSize};"
	bind:this={vm.element}
>
	<div
		class="grid gap-x-1 pe-2 ps-3 sm:pe-3 sm:ps-3 md:gap-x-3
		{isMacos() ? 'ms-[68px]' : ''}
		{isWindows() || (isLinux() && !appStore.isCefEnabled) ? 'me-[100px] sm:me-0' : ''}
		{isMobile()
			? 'grid-cols-[min-content_1fr_min-content_min-content] sm:grid-cols-[min-content_min-content_1fr]'
			: 'grid-cols-[1fr_min-content_min-content] sm:grid-cols-[min-content_1fr]'}"
	>
		{#if isMobile()}
			<Button
				class="pointer-events-auto grid aspect-square h-9 justify-center rounded sm:p-0"
				onclick={vm.handleMenuButton}
			>
				<div class="w-5">
					<Icon type={IconType.Menu} />
				</div>
			</Button>
		{/if}

		<Input
			class="pointer-events-auto h-9 rounded p-0 sm:hidden"
			icon={IconType.Search}
			placeholder="Search..."
			bind:value={filterStore.search}
		/>

		<Button
			class="pointer-events-auto grid aspect-square h-9 justify-center rounded"
			onclick={vm.toggleSort}
		>
			<div class="w-5">
				{#if filterBarStore.sortAsc}
					<Icon type={IconType.SortAsc} />
				{:else}
					<Icon type={IconType.SortDesc} />
				{/if}
			</div>
		</Button>

		{#if isMobile()}
			<Button
				class="pointer-events-auto grid aspect-square h-9 justify-center rounded sm:hidden sm:p-0"
				onclick={vm.handleQueueButton}
			>
				<div class="w-5">
					<Icon type={IconType.Queue} />
				</div>
			</Button>
		{/if}

		<div class="pointer-events-auto hidden h-9 w-full min-w-0 sm:flex sm:items-center sm:gap-x-1">
			{#if musicStore.listType === 'playlist' && playlistStore.isCreating && vm.state.columns < 5}
				<ConfirmCancelButtons
					onconfirm={vm.confirmPlaylistCreation}
					oncancel={vm.cancelPlaylistCreation}
				/>
			{:else}
				<ListModeSelector
					options={vm.tracksOptions}
					selected={musicStore.listType}
					onchange={vm.handleToggleChange}
					iconStyle="width: {vm.iconSize}px;"
				/>
				{#if musicStore.listType === 'playlist' && vm.state.columns < 5}
					<AddPlaylistButton onclick={vm.startPlaylistCreation} />
				{/if}
			{/if}
		</div>
	</div>

	<div class="h-9 px-3 sm:hidden">
		<div class="flex h-9 w-full min-w-0 items-center gap-x-1">
			{#if musicStore.listType === 'playlist' && playlistStore.isCreating}
				<ConfirmCancelButtons
					onconfirm={vm.confirmPlaylistCreation}
					oncancel={vm.cancelPlaylistCreation}
				/>
			{:else}
				<ListModeSelector
					options={vm.tracksOptions}
					selected={musicStore.listType}
					onchange={vm.handleToggleChange}
					iconStyle="width: {vm.iconSize}px;"
				/>
				{#if musicStore.listType === 'playlist'}
					<AddPlaylistButton onclick={vm.startPlaylistCreation} />
				{/if}
			{/if}
		</div>
	</div>
	<div class="hidden sm:block" style={vm.state.columns > 5 ? 'width: 50%;' : ''}>
		{#if musicStore.listType === 'playlist' && playlistStore.isCreating && vm.state.columns >= 5}
			<ConfirmCancelButtons
				onconfirm={vm.confirmPlaylistCreation}
				oncancel={vm.cancelPlaylistCreation}
			/>
		{:else if musicStore.listType === 'playlist' && vm.state.columns >= 5}
			<AddPlaylistButton onclick={vm.startPlaylistCreation} />
		{/if}
	</div>
	<div
		class="hidden sm:grid sm:ps-3
		{isMobile() ? 'sm:pe-3' : 'gap-x-1 sm:grid-cols-[1fr_min-content] md:gap-x-3'}
		{isLinux() && !appStore.isCefEnabled ? 'me-[120px]' : ''}
		{isWindows() ? 'me-[120px]' : ''}"
	>
		<Input
			class="pointer-events-auto h-9 rounded p-0"
			icon={IconType.Search}
			placeholder="Search..."
			bind:value={filterStore.search}
		/>

		{#if isMobile()}
			<Button
				class="pointer-events-auto grid aspect-square h-9 justify-center rounded sm:p-0"
				onclick={vm.handleQueueButton}
			>
				<div class="w-5">
					<Icon type={IconType.Queue} />
				</div>
			</Button>
		{/if}
	</div>
</div>
