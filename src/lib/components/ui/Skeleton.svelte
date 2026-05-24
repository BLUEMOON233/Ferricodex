<script lang="ts">
  type Props = {
    width?: string;
    height?: string;
    radius?: string;
    class?: string;
  };

  let {
    width = "100%",
    height = "1em",
    radius = "var(--radius-xs)",
    class: className = "",
  }: Props = $props();
</script>

<span
  class="skeleton {className}"
  style:width
  style:height
  style:border-radius={radius}
></span>

<style>
  .skeleton {
    display: inline-block;
    background: linear-gradient(
      90deg,
      var(--bg-input-soft) 0%,
      var(--bg-surface-hover) 50%,
      var(--bg-input-soft) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.4s ease-in-out infinite;
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }
</style>
