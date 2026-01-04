<script lang="ts">
	import "./app.css"

	let { children, currentLocation, navigate, account } = $props();

	const tabs = ['PLAY', 'INSTALLATIONS'];

	function getActiveTab() {
		if (currentLocation === '/' || currentLocation === '') return 'PLAY';
		if (currentLocation === '/install') return 'INSTALLATIONS';
		return 'PLAY';
	}

	function handleTabClick(tab: string) {
		if (tab === 'PLAY') navigate('/');
		if (tab === 'INSTALLATIONS') navigate('/install');
	}
</script>

<div class="launcher-container">
	<div class="launcher-window">
		<!-- Top Bar -->
		<div class="top-bar">
			<div class="top-bar-left">
				<span class="launcher-title">Minecraft Launcher</span>
			</div>

			<div class="top-bar-center">
				{#each tabs as tab}
					<button
						class="nav-button"
						class:active={getActiveTab() === tab}
						onclick={() => handleTabClick(tab)}
					>
						{tab}
					</button>
				{/each}
			</div>

			<div class="top-bar-right">
				<div class="user-status">
					<span>playing as <b>{account?.name}</b></span>
					<img
						src={`https://mc-heads.net/avatar/${account?.name}`}
						alt={`${account?.name}'s avatar`}
						class="user-avatar"
					/>
				</div>
			</div>
		</div>

		<!-- Main Content -->
		<div class="main-content">
			{@render children()}
		</div>
	</div>
</div>

<style>
	.launcher-container {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100vw;
		height: 100vh;
		background: linear-gradient(135deg, #e8e8e8 0%, #d4d4d4 100%);
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
	}

	.launcher-window {
		width: 100%;
		height: 100%;
		background: #ffffff;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		position: relative;
	}

	.top-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 24px;
		background: #f5f5f5;
		border-bottom: 1px solid #e0e0e0;
	}

	.top-bar-left {
		flex: 1;
	}

	.launcher-title {
		font-size: 16px;
		font-weight: 600;
		color: #1a1a1a;
	}

	.top-bar-center {
		display: flex;
		gap: 8px;
	}

	.nav-button {
		padding: 8px 20px;
		border-radius: 9999px;
		border: 2px solid #d1d1d1;
		background: #ffffff;
		color: #1a1a1a;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.nav-button:hover {
		background: #e8e8e8;
	}

	.nav-button.active {
		background: #1a1a1a;
		color: #ffffff;
	}

	.top-bar-right {
		flex: 1;
		display: flex;
		justify-content: flex-end;
	}

	.user-status {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 13px;
		color: #666666;
	}

	.user-status b {
		color: #1a1a1a;
	}

	.user-avatar {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		object-fit: cover;
	}

	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
	}

	@media (max-width: 950px) {
		.top-bar {
			flex-direction: column;
			gap: 12px;
		}

		.top-bar-left,
		.top-bar-right {
			flex: 0;
		}

		.user-status span {
			font-size: 11px;
		}
	}
</style>
