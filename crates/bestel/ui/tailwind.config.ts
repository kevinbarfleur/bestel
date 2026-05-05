import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{vue,ts,tsx,js,jsx}'],
  theme: {
    extend: {
      colors: {
        bg: 'var(--color-bg)',
        'bg-primary': 'var(--color-bg-primary)',
        surface: 'var(--color-surface)',
        'surface-light': 'var(--color-surface-light)',
        border: 'var(--color-border)',
        'border-light': 'var(--color-border-light)',
        text: 'var(--color-text)',
        'text-dim': 'var(--color-text-dim)',
        'text-muted': 'var(--color-text-muted)',
        accent: 'var(--color-accent)',
        'accent-light': 'var(--color-accent-light)',
        'accent-dark': 'var(--color-accent-dark)',
        gold: 'var(--color-gold)',
      },
      fontFamily: {
        display: ['Cinzel', 'serif'],
        body: ['Crimson Text', 'serif'],
      },
      borderRadius: {
        runic: 'var(--radius-runic)',
      },
      transitionTimingFunction: {
        smooth: 'cubic-bezier(0.03, 0.98, 0.52, 0.99)',
        runic: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
      },
      zIndex: {
        modal: '10000',
        dropdown: '10001',
        tooltip: '10100',
        toast: '10200',
      },
    },
  },
  plugins: [],
} satisfies Config;
