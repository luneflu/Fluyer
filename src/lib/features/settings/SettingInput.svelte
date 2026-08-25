<script lang="ts">
	import View from '$lib/ui/components/View.svelte';

	interface Props {
		children?: import('svelte').Snippet;
		class?: string;
	}

	let { class: className, children }: Props = $props();
</script>

<div class="w-full px-5 {className}">
	<View class="my-3 w-full rounded bg-transparent"
		glassShineSize="xs">
		{@render children?.()}
	</View>
</div>
