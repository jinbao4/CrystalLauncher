<script lang="ts">
	import { cn } from '$lib/utils';

	interface CardContentProps {
		class?: string;
		children: import('svelte').Snippet;
	}
	let { class: className = '', children }: CardContentProps = $props();
</script>

<div class={cn('p-6 pt-0', className)}>
	{@render children()}
</div>

