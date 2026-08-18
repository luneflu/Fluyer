<script lang="ts">
	import Button from '$lib/ui/components/Button.svelte';
	import { useIntro } from '../viewmodels/useIntro.svelte';
	import { isAndroid, isDesktop } from '$lib/platform';

	const vm = useIntro();
</script>

<div
	class="anim grid h-full items-center justify-items-center {vm.animatedClasses}"
	onanimationend={() => vm.onAnimationEnd(vm.animatedClasses)}
>
	<div class="grid justify-items-center px-5 text-center lg:px-0">
		<p class="text-2xl font-bold md:text-3xl">Let's Set Up Your Music</p>
		<p class="mt-3 text-lg md:text-xl">
			{#if isAndroid()}
				To get started, allow access and select your music folder.
			{:else if isDesktop()}
				Select your music folder to get started.
			{/if}
		</p>
		<Button class="mt-5 w-fit rounded p-2" onclick={vm.requestAction}>
			{#if isAndroid()}
				Allow Access & Choose Folder
			{:else if isDesktop()}
				Choose Folder
			{/if}
		</Button>
	</div>
</div>
