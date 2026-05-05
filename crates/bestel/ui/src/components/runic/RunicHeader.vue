<script setup lang="ts">
interface Props {
  title: string;
  subtitle?: string;
  centered?: boolean;
  attached?: boolean;
}

withDefaults(defineProps<Props>(), {
  subtitle: undefined,
  centered: true,
  attached: false,
});
</script>

<template>
  <div
    class="runic-header-wrapper"
    :class="{ 'runic-header-wrapper--attached': attached }"
  >
    <div
      class="runic-header"
      :class="{
        'runic-header--centered': centered,
        'runic-header--attached': attached,
      }"
    >
      <div class="runic-header__accent runic-header__accent--left"></div>
      <div class="runic-header__accent runic-header__accent--right"></div>

      <div class="runic-header__content">
        <h1 class="runic-header__title">
          <span class="runic-header__rune">◆</span>
          {{ title }}
          <span class="runic-header__rune">◆</span>
        </h1>
        <p v-if="subtitle" class="runic-header__subtitle">{{ subtitle }}</p>
      </div>

      <div class="runic-header__edge"></div>
    </div>
  </div>
</template>

<style scoped>
.runic-header-wrapper {
  position: relative;
}

.runic-header-wrapper--attached {
  padding: 0 12px;
  margin-bottom: -1px;
}

@media (min-width: 640px) {
  .runic-header-wrapper--attached {
    padding: 0 16px;
  }
}

@media (min-width: 768px) {
  .runic-header-wrapper--attached {
    padding: 0 20px;
  }
}

.runic-header {
  position: relative;
  padding: 1rem 1.25rem 0.85rem;
  background: var(--paper-shade);
  border-radius: 4px 7px 0 0 / 6px 4px 0 0;
  border: 1.4px solid var(--ink-soft);
  border-bottom: 1.4px dashed var(--ink-faint);
}

.runic-header--centered {
  text-align: center;
}

.runic-header--attached {
  border-radius: 6px 6px 0 0;
}

.runic-header__accent {
  position: absolute;
  top: 10px;
  width: 35px;
  height: 1px;
  pointer-events: none;
}

.runic-header__accent--left {
  left: 10px;
  background: linear-gradient(to right, var(--amber), transparent);
  opacity: 0.6;
}

.runic-header__accent--right {
  right: 10px;
  background: linear-gradient(to left, var(--amber), transparent);
  opacity: 0.6;
}

.runic-header__content {
  position: relative;
  z-index: 1;
}

.runic-header__title {
  font-family: var(--hand-display);
  font-size: 1.5rem;
  font-weight: 700;
  letter-spacing: 0.02em;
  color: var(--ink);
  margin: 0;
  white-space: nowrap;
}

@media (max-width: 480px) {
  .runic-header__title {
    font-size: 1.125rem;
  }
  .runic-header__rune {
    margin: 0 0.5rem;
  }
}

.runic-header__rune {
  display: inline-block;
  font-size: 0.55rem;
  color: var(--amber);
  margin: 0 0.6rem;
  vertical-align: middle;
  opacity: 0.6;
}

.runic-header__subtitle {
  font-family: var(--hand);
  font-size: 1rem;
  font-style: italic;
  color: var(--ink-soft);
  margin: 0.4rem 0 0;
}

.runic-header__edge {
  display: none;
}

@media (min-width: 640px) {
  .runic-header {
    padding: 1.5rem 2rem 1.25rem;
  }
  .runic-header__title {
    font-size: 1.75rem;
  }
  .runic-header__subtitle {
    font-size: 1.1875rem;
  }
  .runic-header__accent {
    width: 45px;
  }
}

@media (min-width: 768px) {
  .runic-header__title {
    font-size: 2rem;
    letter-spacing: 0.1em;
  }
  .runic-header__subtitle {
    font-size: 1.25rem;
  }
}
</style>
