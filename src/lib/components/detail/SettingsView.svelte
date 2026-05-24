<script lang="ts">
  import { FolderOpen, Languages, Minus, Monitor, Moon, Plus, RotateCcw, Sun } from "@lucide/svelte";
  import { Badge, IconButton, Kbd } from "$lib/components/ui";
  import FactsGrid from "./FactsGrid.svelte";
  import { theme, type ThemePreference } from "$lib/theme.svelte";
  import { resetZoom, zoom, zoomIn, zoomOut } from "$lib/zoom.svelte";
  import { modKeyLabel } from "$lib/platform";
  import { i18n, t, type LocalePreference } from "$lib/i18n.svelte";
  import type { CodexHomeStatus } from "$lib/codex";

  type Props = {
    codexHome: CodexHomeStatus | null;
    onOpenCodexHome: () => void;
  };

  let { codexHome, onOpenCodexHome }: Props = $props();

  const facts = $derived(
    codexHome
      ? [
          {
            label: t("settings.codexHome.facts.path"),
            value: codexHome.path,
            mono: true,
            title: codexHome.path,
            wide: true,
          },
          {
            label: t("settings.codexHome.facts.source"),
            value:
              codexHome.source === "env"
                ? t("settings.codexHome.source.env")
                : t("settings.codexHome.source.default"),
          },
          {
            label: t("settings.codexHome.facts.exists"),
            value: codexHome.exists ? t("common.yes") : t("common.no"),
          },
          {
            label: t("settings.codexHome.facts.stateDb"),
            value: codexHome.stateDbExists ? t("common.available") : t("common.missing"),
          },
        ]
      : [],
  );

  const themeOptions = $derived<
    { value: ThemePreference; label: string; icon: typeof Sun }[]
  >([
    { value: "system", label: t("settings.appearance.system"), icon: Monitor },
    { value: "light", label: t("settings.appearance.light"), icon: Sun },
    { value: "dark", label: t("settings.appearance.dark"), icon: Moon },
  ]);

  const localeOptions = $derived<{ value: LocalePreference; label: string }[]>([
    { value: "auto", label: t("settings.language.system") },
    { value: "en", label: t("settings.language.english") },
    { value: "zh", label: t("settings.language.chinese") },
  ]);
</script>

<section class="page">
  <header class="head">
    <span class="eyebrow"><Badge variant="muted">{t("settings.preferencesBadge")}</Badge></span>
    <h2>{t("settings.title")}</h2>
    <p>{t("settings.description")}</p>
  </header>

  <section class="card">
    <header class="card-head">
      <h3>{t("settings.appearance.title")}</h3>
    </header>
    <p class="card-text">{t("settings.appearance.description")}</p>
    <div class="theme-options" role="radiogroup" aria-label={t("settings.appearance.title")}>
      {#each themeOptions as option (option.value)}
        {@const Icon = option.icon}
        <button
          type="button"
          class="theme-option"
          class:active={theme.preference === option.value}
          role="radio"
          aria-checked={theme.preference === option.value}
          onclick={() => theme.set(option.value)}
        >
          <Icon size={15} />
          <span>{option.label}</span>
        </button>
      {/each}
    </div>
  </section>

  <section class="card">
    <header class="card-head">
      <h3>{t("settings.language.title")}</h3>
      <span class="card-meta"><Languages size={14} /></span>
    </header>
    <p class="card-text">{t("settings.language.description")}</p>
    <div class="theme-options" role="radiogroup" aria-label={t("settings.language.title")}>
      {#each localeOptions as option (option.value)}
        <button
          type="button"
          class="theme-option"
          class:active={i18n.preference === option.value}
          role="radio"
          aria-checked={i18n.preference === option.value}
          onclick={() => i18n.set(option.value)}
        >
          <span>{option.label}</span>
        </button>
      {/each}
    </div>
  </section>

  <section class="card">
    <header class="card-head">
      <h3>{t("settings.zoom.title")}</h3>
      <span class="card-meta">
        <Kbd>{modKeyLabel}</Kbd><Kbd>+</Kbd>
        <span class="card-meta-sep">/</span>
        <Kbd>{modKeyLabel}</Kbd><Kbd>−</Kbd>
        <span class="card-meta-sep">/</span>
        <Kbd>{modKeyLabel}</Kbd><Kbd>0</Kbd>
      </span>
    </header>
    <p class="card-text">{t("settings.zoom.description")}</p>
    <div class="zoom-row">
      <IconButton
        aria-label={t("settings.zoom.out")}
        title={t("settings.zoom.out")}
        disabled={zoom.atMin}
        onclick={zoomOut}
      >
        <Minus size={15} />
      </IconButton>
      <div class="zoom-value" aria-live="polite">{zoom.percent}%</div>
      <IconButton
        aria-label={t("settings.zoom.in")}
        title={t("settings.zoom.in")}
        disabled={zoom.atMax}
        onclick={zoomIn}
      >
        <Plus size={15} />
      </IconButton>
      <IconButton
        aria-label={t("settings.zoom.reset")}
        title={t("settings.zoom.reset")}
        disabled={zoom.isDefault}
        onclick={resetZoom}
      >
        <RotateCcw size={14} />
      </IconButton>
    </div>
  </section>

  <section class="card">
    <header class="card-head">
      <h3>{t("settings.codexHome.title")}</h3>
      {#if codexHome?.exists}
        <IconButton
          aria-label={t("action.openCodexHome")}
          title={t("action.openCodexHome")}
          onclick={onOpenCodexHome}
        >
          <FolderOpen size={15} />
        </IconButton>
      {/if}
    </header>

    {#if codexHome}
      <FactsGrid {facts} columns={2} />
    {:else}
      <p class="card-text">{t("settings.codexHome.unavailable")}</p>
    {/if}
  </section>

  <section class="card">
    <header class="card-head">
      <h3>{t("settings.about.title")}</h3>
    </header>
    <p class="card-text">{t("settings.about.description")}</p>
  </section>
</section>

<style>
  .page {
    display: grid;
    gap: 16px;
    width: 100%;
    max-width: 720px;
    margin: 0 auto;
  }

  .head h2 {
    margin-top: 6px;
    color: var(--fg);
    font-size: 18px;
    font-weight: 600;
    line-height: 1.3;
  }

  .head p {
    margin-top: 4px;
    color: var(--fg-tertiary);
    font-size: 12px;
  }

  .card {
    display: grid;
    gap: 10px;
    padding: 14px 16px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--separator);
  }

  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .card h3 {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .card-text {
    color: var(--fg-secondary);
    font-size: 13px;
    line-height: 1.55;
  }

  .card-meta {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: var(--fg-tertiary);
    font-size: 11px;
  }

  .card-meta-sep {
    margin: 0 2px;
    color: var(--fg-tertiary);
  }

  .zoom-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px;
    border-radius: var(--radius-md);
    background: var(--bg-input-soft);
    width: fit-content;
  }

  .zoom-value {
    min-width: 56px;
    padding: 0 6px;
    color: var(--fg);
    font-size: 12px;
    font-weight: 600;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .theme-options {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
    padding: 4px;
    border-radius: var(--radius-md);
    background: var(--bg-input-soft);
  }

  .theme-option {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--fg-secondary);
    background: transparent;
    font-size: 12px;
    font-weight: 500;
    transition:
      background-color 80ms ease,
      color 80ms ease,
      box-shadow 80ms ease;
  }

  .theme-option:hover {
    color: var(--fg);
  }

  .theme-option.active {
    color: var(--fg);
    background: var(--bg-surface);
    box-shadow: var(--shadow-sm);
  }
</style>
