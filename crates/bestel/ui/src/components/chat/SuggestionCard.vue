<script setup lang="ts">
import RunicButton from '../runic/RunicButton.vue';
import RunicIcon from '../runic/RunicIcon.vue';

defineProps<{
  buildName: string;
  dismissed?: boolean;
}>();
defineEmits<{
  (e: 'create'): void;
  (e: 'later'): void;
  (e: 'dismiss'): void;
}>();
</script>

<template>
  <div v-if="dismissed" class="suggestion-card suggestion-card--collapsed">
    · build sheet suggestion hidden for 7 days ·
  </div>
  <div v-else class="suggestion-card">
    <div class="suggestion-card__header">
      <span class="suggestion-card__icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 18h6" />
          <path d="M10 21h4" />
          <path d="M12 2a7 7 0 0 0-4 12.7c.7.6 1 1.5 1 2.3v.5h6V17c0-.8.3-1.7 1-2.3A7 7 0 0 0 12 2z" />
        </svg>
      </span>
      <div class="suggestion-card__body">
        <div class="suggestion-card__title">Considering a deeper audit?</div>
        <div class="suggestion-card__text">
          Create a build sheet for <strong>{{ buildName }}</strong> so future chats can reuse the analysis without re-asking the same questions.
        </div>
      </div>
      <button class="suggestion-card__close" type="button" aria-label="Dismiss for 7 days" @click="$emit('dismiss')">
        <RunicIcon name="close" :size="12" />
      </button>
    </div>
    <div class="suggestion-card__actions">
      <RunicButton variant="primary" icon="plus" @click="$emit('create')">Create build sheet</RunicButton>
      <button type="button" class="suggestion-card__link" @click="$emit('later')">Maybe later</button>
      <span class="suggestion-card__spacer" />
      <span class="suggestion-card__cooldown">Dismissal remembered for 7 days</span>
    </div>
  </div>
</template>

<style scoped>
.suggestion-card {
  max-width: 540px;
  padding: 14px 18px;
  background: var(--paper);
  border: 1px dashed var(--ink-faint);
  border-radius: 5px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 6px;
}
.suggestion-card--collapsed {
  font-size: 12px;
  color: var(--ink-faint);
  font-family: var(--label);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  padding: 4px 0;
  border: none;
  background: transparent;
  margin-top: 6px;
  text-align: left;
}
.suggestion-card__header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}
.suggestion-card__icon {
  width: 28px;
  height: 28px;
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--amber-soft);
  border-radius: 4px;
  color: var(--amber);
  background: var(--amber-glow);
}
.suggestion-card__icon svg {
  width: 14px;
  height: 14px;
}
.suggestion-card__body {
  flex: 1;
}
.suggestion-card__title {
  font-size: 15px;
  font-weight: 600;
  color: var(--ink);
}
.suggestion-card__text {
  font-size: 14px;
  color: var(--ink-soft);
  margin-top: 4px;
  line-height: 1.5;
}
.suggestion-card__close {
  width: 22px;
  height: 22px;
  padding: 0;
  background: transparent;
  color: var(--ink-faint);
  border: none;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.suggestion-card__close:hover {
  color: var(--ink);
}
.suggestion-card__actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
.suggestion-card__link {
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  font-family: var(--hand);
  font-size: 13px;
  font-weight: 500;
  color: var(--amber);
  text-decoration: underline;
  text-decoration-style: dotted;
  text-underline-offset: 3px;
}
.suggestion-card__spacer {
  flex: 1;
}
.suggestion-card__cooldown {
  font-size: 12px;
  color: var(--ink-faint);
}
</style>
