//! Navbar styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Navigation Bar (Minimalist Glassmorphism)
   ========================================================================== */
.navbar {
  position: sticky;
  top: 0;
  z-index: 100;
  background: rgba(8, 9, 10, 0.8);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border-bottom: 1px solid var(--border-subtle);
  padding: 18px 0;
  transition: border-color 0.3s ease;
}

.nav-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.nav-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  text-decoration: none;
  color: var(--text-primary);
}

.brand-logo-img {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  object-fit: contain;
  box-shadow: 0 2px 10px rgba(0, 160, 255, 0.25);
  transition: transform 0.2s ease;
}

.nav-brand:hover .brand-logo-img {
  transform: scale(1.05);
}

.brand-badge {
  width: 32px;
  height: 32px;
  background: linear-gradient(180deg, #ffffff 0%, #cbd5e1 100%);
  color: #000;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  font-size: 1rem;
  box-shadow: 0 2px 10px rgba(255, 255, 255, 0.15);
}

.brand-text {
  font-weight: 800;
  font-size: 1.25rem;
  letter-spacing: -0.5px;
}

.accent-dot {
  color: var(--accent-titanium);
}

.brand-chip {
  font-family: var(--font-mono);
  font-size: 0.65rem;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-subtle);
  padding: 3px 8px;
  border-radius: 99px;
  color: var(--text-secondary);
  letter-spacing: 0.5px;
}

.nav-links {
  display: flex;
  gap: 32px;
}

.nav-links a {
  color: var(--text-secondary);
  text-decoration: none;
  font-size: 0.9rem;
  font-weight: 500;
  transition: color 0.2s ease;
}

.nav-links a:hover {
  color: var(--text-primary);
}

.nav-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}"####
}
