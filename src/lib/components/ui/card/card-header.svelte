<script lang="ts">
	import { cn } from '$lib/utils';

	interface CardHeaderProps {
		class?: string;
		children: import('svelte').Snippet;
	}
	let { class: className = '', children }: CardHeaderProps = $props();
</script>

<div class={cn('flex flex-col space-y-1.5 p-6', className)}>
	{@render children()}
</div>

