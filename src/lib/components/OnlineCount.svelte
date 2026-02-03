<script lang="ts">
    import Skins from "./Skins.svelte";

    let { account } = $props();
    const loggedIn = $derived(!!account);
</script>

<div class="top-bar-right">
    {#if loggedIn}
        <button class="account-card">
            <div class="text">
                <span class="label">Signed in as</span>
                <span class="username">{account.name}</span>
            </div>

            <div class="avatar">
                <Skins username={account.name} width={300} height={400} />
            </div>
        </button>
    {:else}
        <button class="login-btn" on:click={() => (window.location.href = "/link")}>
            Log In
        </button>
    {/if}
</div>
<style>
    .top-bar-right {
        display: flex;
        justify-content: flex-end;
        padding: 12px;
        position: relative;
        z-index: 50;
    }

    .account-card {
        display: flex;
        align-items: center;
        gap: 12px;

        height: 60px;
        width: 220px;
        padding: 0 14px 0 18px;
        margin-top: 20px;

        background: rgba(20, 20, 25, 0.9);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 12px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);

        cursor: pointer;
        transition: transform 0.15s ease, background 0.15s ease, border-color 0.15s ease;
    }

    .account-card:hover {
        transform: translateY(-2px);
        background: rgba(30, 30, 35, 0.95);
        border-color: rgba(255, 255, 255, 0.2);
    }

    .account-card:active {
        transform: translateY(0);
    }

    /* === TEXT === */
    .text {
        display: flex;
        flex-direction: column;
    }

    .label {
        font-size: 10px;
        font-weight: 600;
        color: #7d8590;
        text-transform: uppercase;
        letter-spacing: 0.8px;
    }

    .username {
        font-size: 20px;
        font-weight: 800;
        color: #fff;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    /* === AVATAR === */
    .avatar {
        width: 72px;
        display: flex;
        align-items: flex-end;
        justify-content: center;

        transform: scale(0.42) translate(12px, -15px);
        transform-origin: bottom center;

        pointer-events: none;
        image-rendering: pixelated;
    }

    /* === LOGIN BUTTON === */
    .login-btn {
        height: 48px;
        padding: 0 28px;

        background: #2563eb;
        color: white;
        border: none;
        border-radius: 8px;

        font-weight: 700;
        cursor: pointer;
        transition: background 0.15s ease;
    }

    .login-btn:hover {
        background: #1d4ed8;
    }

</style>