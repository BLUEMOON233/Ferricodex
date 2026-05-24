<script lang="ts">
  import { CheckCircle2, AlertTriangle, AlertCircle, Info, X } from "@lucide/svelte";
  import { dismiss, getToasts, type Toast } from "$lib/toast.svelte";

  const toasts = $derived<Toast[]>(getToasts());

  function iconFor(variant: Toast["variant"]) {
    if (variant === "success") return CheckCircle2;
    if (variant === "warning") return AlertTriangle;
    if (variant === "error") return AlertCircle;
    return Info;
  }
</script>

<div class="toaster" role="status" aria-live="polite" aria-relevant="additions">
  {#each toasts as t (t.id)}
    {@const Icon = iconFor(t.variant)}
    <div class="toast toast-{t.variant}">
      <span class="toast-icon" aria-hidden="true">
        <Icon size={16} />
      </span>
      <div class="toast-body">
        <strong>{t.title}</strong>
        {#if t.description}
          <span>{t.description}</span>
        {/if}
      </div>
      <button
        class="toast-close"
        type="button"
        aria-label="Dismiss"
        onclick={() => dismiss(t.id)}
      >
        <X size={14} />
      </button>
    </div>
  {/each}
</div>

<style>
  .toaster {
    position: fixed;
    z-index: 9000;
    display: grid;
    gap: 8px;
    right: 16px;
    bottom: 16px;
    width: min(360px, calc(100vw - 32px));
    pointer-events: none;
  }

  .toast {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: flex-start;
    gap: 10px;
    padding: 11px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    color: var(--fg);
    background: var(--bg-elevated);
    box-shadow: var(--shadow-lg);
    pointer-events: auto;
    animation: toast-in 160ms ease-out;
  }

  .toast-icon {
    display: grid;
    place-items: center;
    margin-top: 1px;
    color: var(--fg-secondary);
  }

  .toast-success .toast-icon {
    color: var(--success);
  }

  .toast-warning .toast-icon {
    color: var(--warning);
  }

  .toast-error .toast-icon {
    color: var(--danger);
  }

  .toast-info .toast-icon {
    color: var(--accent);
  }

  .toast-body {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .toast-body strong {
    font-size: 13px;
    font-weight: 600;
    line-height: 1.4;
  }

  .toast-body span {
    color: var(--fg-secondary);
    font-size: 12px;
    line-height: 1.45;
  }

  .toast-close {
    display: grid;
    width: 22px;
    height: 22px;
    place-items: center;
    border-radius: var(--radius-xs);
    color: var(--fg-tertiary);
    background: transparent;
  }

  .toast-close:hover {
    color: var(--fg);
    background: var(--bg-surface-hover);
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
