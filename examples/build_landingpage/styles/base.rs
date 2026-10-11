//! Base styles: CSS variables, reset, ambient glow, container

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Kora Landing Page — Titanium & Obsidian Monochrome (Apple Pro / Linear.app)
   ========================================================================== */

:root {
  --bg-black: #08090a;
  --bg-deep: #0c0d0e;
  --bg-card: rgba(255, 255, 255, 0.025);
  --bg-card-hover: rgba(255, 255, 255, 0.05);
  --bg-card-elevated: rgba(18, 20, 24, 0.85);

  --border-subtle: rgba(255, 255, 255, 0.07);
  --border-strong: rgba(255, 255, 255, 0.15);
  --border-focus: rgba(255, 255, 255, 0.35);

  --text-primary: #f8fafc;
  --text-secondary: #94a3b8;
  --text-muted: #64748b;

  --accent-silver: #e2e8f0;
  --accent-titanium: #cbd5e1;
  --accent-green: #10b981;
  --accent-rose: #f43f5e;

  --font-main: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
}

*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html {
  scroll-behavior: smooth;
  color-scheme: dark;
}

body {
  background-color: var(--bg-black);
  color: var(--text-primary);
  font-family: var(--font-main);
  line-height: 1.6;
  overflow-x: hidden;
  position: relative;
  min-height: 100vh;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Subtle Titanium Ambient Sheen (No aggressive RGB colors) */
.ambient-glow {
  position: fixed;
  border-radius: 50%;
  filter: blur(160px);
  pointer-events: none;
  z-index: 0;
  opacity: 0.07;
}

.glow-cyan {
  width: 700px;
  height: 700px;
  background: radial-gradient(circle, #ffffff, transparent 65%);
  top: -200px;
  left: 30%;
  opacity: 0.06;
}

.glow-crimson {
  width: 800px;
  height: 800px;
  background: radial-gradient(circle, #94a3b8, transparent 70%);
  top: 35%;
  right: -250px;
  opacity: 0.04;
}

.glow-blue {
  width: 900px;
  height: 900px;
  background: radial-gradient(circle, #cbd5e1, transparent 70%);
  bottom: 5%;
  left: -250px;
  opacity: 0.04;
}

.container {
  width: 100%;
  max-width: 1160px;
  margin: 0 auto;
  padding: 0 24px;
  position: relative;
  z-index: 1;
}"####
}
