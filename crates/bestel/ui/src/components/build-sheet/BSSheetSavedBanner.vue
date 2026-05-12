<script setup lang="ts">
defineProps<{
  /** Sheet name persisted to the DB row — surfaced in a tooltip so the
   *  user can confirm at-a-glance which sheet was saved without going to
   *  the sidebar card. */
  name: string;
}>();
</script>

<template>
  <div class="bs-saved" :title="`Sheet name: ${name}`">
    <span class="bs-saved__check" aria-hidden="true">✓</span>
    Build Sheet saved · Bestel will reuse it in every chat about this build
  </div>
</template>

<style scoped>
.bs-saved {
  align-self: center;
  margin: 8px 0;
  padding: 8px 14px;
  border: 1px solid var(--good);
  border-radius: 16px;
  background: rgba(84, 124, 74, 0.08);
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--good);
  font-weight: 700;
  white-space: normal;
  text-align: center;
}
.bs-saved__check {
  font-family: var(--hand);
  font-size: 13px;
  letter-spacing: 0;
  line-height: 1;
}
</style>
