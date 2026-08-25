<script lang="ts">
	import type { IconType } from '$lib/ui/icon/types';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import Button from '$lib/ui/components/Button.svelte';

	interface Props {
		onclick?: () => void;
		icon?: IconType;
		label: string;
	}

	const { onclick, label, icon }: Props = $props();
</script>

<div class="w-full px-5">
	<Button
		class="my-3 w-full rounded px-3 py-2 text-start bg-transparent
        {icon ? 'grid grid-cols-[min-content_auto] items-center gap-2' : ''}"
		{onclick}
	>
		{#if icon}
			<div class="w-5"><Icon type={icon} /></div>
		{/if}
		<div class="text-start">{label}</div>
	</Button>
</div>
