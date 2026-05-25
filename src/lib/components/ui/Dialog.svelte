<script lang="ts">
  import type { Snippet } from "svelte";

  type Tone = "neutral" | "danger";

  type Props = {
    open: boolean;
    title: string;
    description?: string;
    tone?: Tone;
    closeOnBackdrop?: boolean;
    onclose?: () => void;
    children?: Snippet;
    actions?: Snippet;
  };

  let {
    open = $bindable(false),
    title,
    description,
    tone = "neutral",
    closeOnBackdrop = true,
    onclose,
    children,
    actions,
  }: Props = $props();

  let dialogEl: HTMLDialogElement | undefined = $state();

  $effect(() => {
    const el = dialogEl;
    if (!el) {
      return;
    }

    if (open && !el.open) {
      el.showModal();
    } else if (!open && el.open) {
      el.close();
    }
  });

  function handleClose() {
    open = false;
    onclose?.();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (!closeOnBackdrop || event.target !== dialogEl) {
      return;
    }

    handleClose();
  }
</script>

<dialog
  bind:this={dialogEl}
  class="dialog tone-{tone}"
  onclose={handleClose}
  onclick={handleBackdropClick}
>
  {#if open}
    <div class="dialog-card">
      <header class="dialog-header">
        <h2>{title}</h2>
        {#if description}
          <p>{description}</p>
        {/if}
      </header>

      {#if children}
        <div class="dialog-body">
          {@render children()}
        </div>
      {/if}

      {#if actions}
        <footer class="dialog-actions">
          {@render actions()}
        </footer>
      {/if}
    </div>
  {/if}
</dialog>

<style>
  .dialog {
    width: min(440px, calc(100vw - 32px));
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 64px);
    box-sizing: border-box;
    margin: auto;
    padding: 0;
    border: 0;
    border-radius: var(--radius-xl);
    color: var(--fg);
    background: var(--bg-elevated);
    box-shadow: var(--shadow-dialog);
    overflow: hidden;
  }

  .dialog[open] {
    animation: dialog-in 140ms ease-out;
  }

  .dialog-card {
    display: grid;
    gap: 14px;
    min-width: 0;
    max-width: 100%;
    max-height: calc(100vh - 64px);
    box-sizing: border-box;
    overflow: auto;
    padding: 20px 22px 18px;
  }

  .dialog-header,
  .dialog-body,
  .dialog-actions {
    min-width: 0;
    max-width: 100%;
  }

  .dialog-header h2 {
    font-size: 15px;
    font-weight: 600;
    line-height: 1.35;
  }

  .tone-danger .dialog-header h2 {
    color: var(--fg);
  }

  .dialog-header p {
    margin-top: 6px;
    color: var(--fg-secondary);
    font-size: 13px;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }

  .dialog-body {
    color: var(--fg-secondary);
    font-size: 13px;
    line-height: 1.55;
  }

  .dialog-body :global(p) {
    margin-bottom: 8px;
  }

  .dialog-body :global(p:last-child) {
    margin-bottom: 0;
  }

  .dialog-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  .dialog-actions :global(.btn) {
    max-width: 100%;
  }

  @media (max-width: 520px) {
    .dialog-card {
      padding: 18px 18px 16px;
    }

    .dialog-actions {
      justify-content: stretch;
    }

    .dialog-actions :global(.btn) {
      flex: 1 1 auto;
    }
  }

  @keyframes dialog-in {
    from {
      opacity: 0;
      transform: translateY(4px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
