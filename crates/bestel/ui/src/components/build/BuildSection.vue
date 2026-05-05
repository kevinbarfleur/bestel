<script setup lang="ts">
defineProps<{ title: string; subtitle?: string }>();
</script>

<template>
  <section class="build-section">
    <header class="build-section__head">
      <span class="build-section__bar" />
      <h3 class="build-section__title">{{ title }}</h3>
      <span class="build-section__bar" />
    </header>
    <p v-if="subtitle" class="build-section__subtitle">{{ subtitle }}</p>
    <div class="build-section__body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.build-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.build-section__head {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}

.build-section__bar {
  flex: 1;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(175, 96, 37, 0.35), transparent);
}

.build-section__title {
  margin: 0;
  font-family: 'Cinzel', serif;
  font-size: 0.78rem;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--color-gold, #c9a227);
  text-shadow: 0 0 6px rgba(201, 162, 39, 0.18);
}

.build-section__subtitle {
  margin: 0;
  font-family: 'Cormorant Garamond', serif;
  font-style: italic;
  font-size: 0.85rem;
  color: var(--color-text-dim, #998);
  text-align: center;
}

.build-section__body {
  margin-top: 0.25rem;
}
</style>
