<script lang="ts">
    import Skins from "./Skins.svelte";
    import { Settings, X } from "lucide-svelte";
    import { fly, fade } from "svelte/transition";

    let { account, foo } = $props();
    let loggedIn = $derived(account != null);
    let showUserPanel = $state(false);

    function toggleUserPanel() {
        showUserPanel = !showUserPanel;
    }

    function closePanel() {
        showUserPanel = false;
        if (foo) foo();
    }
</script>

{#snippet microsoftLogo()}
    <svg width="20" height="20" viewBox="0 0 23 23" fill="none">
        <path d="M0 0h11v11H0z" fill="#f25022" />
        <path d="M12 0h11v11H12z" fill="#00a4ef" />
        <path d="M0 12h11v11H0z" fill="#7fba00" />
        <path d="M12 12h11v11H12z" fill="#ffb900" />
    </svg>
{/snippet}

{#snippet addAccountBtn(text)}
    <button
        class="group flex w-full cursor-pointer items-center gap-3 rounded-lg border border-green-700 bg-green-600 px-4 py-3.5 text-left text-sm font-semibold text-white shadow-sm transition-all hover:-translate-y-0.5 hover:bg-green-700 hover:shadow-lg hover:shadow-green-600/30"
    >
        <div class="flex h-5 w-5 shrink-0 items-center justify-center">
            {@render microsoftLogo()}
        </div>
        <span>{text}</span>
    </button>
{/snippet}

<div class="relative z-50 flex items-center justify-end gap-3 px-6 py-3">
    <button
        class="mt-4 flex h-[60px] w-[60px] cursor-pointer items-center justify-center rounded-xl border border-white/10 bg-neutral-950/70 text-white shadow-lg backdrop-blur-md transition-all hover:-translate-y-0.5 hover:border-white/20 hover:bg-neutral-900/80"
        onclick={() => {}}
        aria-label="Settings"
    >
        <Settings size={20} />
    </button>

    {#if loggedIn}
        <button
            onclick={toggleUserPanel}
            class="relative mt-4 flex h-[60px] w-[220px] cursor-pointer items-center justify-between overflow-visible rounded-xl border border-white/20 bg-gradient-to-br from-green-600 to-green-700 px-5 pr-2.5 text-white shadow-lg transition-all hover:-translate-y-0.5 hover:border-white/30"
        >
            <div class="z-10 flex flex-col items-start text-left">
                <span
                    class="text-[9px] font-medium italic tracking-wider text-neutral-400"
                >
                    Playing as
                </span>
                <span class="text-lg font-bold tracking-tight text-white">
                    {account.name}
                </span>
            </div>

            <div class="relative flex h-full w-[70px] items-end justify-center">
                <div
                    class="pointer-events-none absolute -right-[120px] bottom-0 z-10 origin-bottom scale-[0.4] [image-rendering:pixelated]"
                >
                    <Skins username={account.name} width={300} height={400} />
                    </div>
                </div>
        </button>

        {#if showUserPanel}
            <button
                onclick={closePanel}
                class="fixed inset-0 z-[900] h-screen w-screen cursor-default bg-black/70 backdrop-blur-sm"
                transition:fade={{ duration: 200 }}
                aria-label="Close panel"
            ></button>

            <div
                class="fixed right-0 top-0 z-[901] flex h-screen w-full max-w-xl translate-x-2 flex-col border-l border-neutral-800 bg-neutral-900 shadow-2xl"
                transition:fly={{ x: 400, duration: 300, opacity: 1 }}
            >
                <div
                    class="flex h-[60px] shrink-0 items-center justify-between border-b border-neutral-800 px-5"
                >
                    <h2 class="text-sm font-bold tracking-wide text-white">
                        ACCOUNT SWITCHER
                    </h2>
                    <button
                        onclick={closePanel}
                        class="flex cursor-pointer items-center justify-center rounded p-1.5 text-white/40 transition-all hover:bg-white/5 hover:text-white/80"
                        aria-label="Close"
                    >
                        <X size={20} />
                    </button>
                </div>

                <div class="flex flex-col gap-3 overflow-y-auto p-5">
                    <div
                        class="flex items-center gap-3 rounded-lg border border-neutral-800 bg-neutral-950 p-4"
                    >
                        <div
                            class="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-md border border-neutral-800 bg-neutral-900 [image-rendering:pixelated]"
                        >
                            <Skins
                                username={account.name}
                                width={64}
                                height={64}
                            />
                        </div>

                        <div class="flex min-w-0 flex-1 flex-col gap-0.5">
                            <span
                                class="text-[11px] font-semibold uppercase tracking-wide text-green-400"
                            >
                                Currently using
                            </span>
                            <span
                                class="truncate text-[15px] font-semibold text-white"
                            >
                                {account.name}
                            </span>
                        </div>

                        <button
                            class="flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded border border-neutral-800 bg-transparent text-white/30 transition-all hover:border-red-500/30 hover:bg-red-500/10 hover:text-red-500"
                            aria-label="Remove account"
                        >
                            <X size={16} />
                        </button>
                    </div>

                    <div class="mt-2 flex flex-col gap-3">
                        {@render addAccountBtn("+ Microsoft Account")}
                        {@render addAccountBtn(
                            "+ Microsoft Account (via Link)",
                        )}
                    </div>
                </div>
            </div>
        {/if}
    {:else}
        <button
            onclick={() => (window.location.href = "/link")}
            class="mt-4 h-12 cursor-pointer rounded-lg border-none bg-blue-600 px-7 font-bold text-white shadow-lg shadow-blue-600/30 transition-colors hover:bg-blue-700"
        >
            Log In
        </button>
    {/if}
</div>
