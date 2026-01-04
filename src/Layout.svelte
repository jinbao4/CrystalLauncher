<script lang="ts">
	let { children } = $props();
</script>

<svelte:head>
	<style>
		body {
			margin: 0;
		}
	</style>
</svelte:head>

<div class="flex min-h-screen min-w-[320px] items-center justify-center">
	<div class="w-full max-w-[1280px] px-8 text-center">
		{@render children()}
	</div>
</div>
