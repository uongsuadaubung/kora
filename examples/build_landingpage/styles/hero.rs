//! Hero section styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Hero Section
   ========================================================================== */
.hero {
  padding: 130px 0 100px;
  text-align: center;
}

.hero-tag {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-subtle);
  padding: 6px 16px;
  border-radius: 99px;
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 1.2px;
  color: var(--text-secondary);
  margin-bottom: 32px;
}

.pulse-indicator {
  width: 6px;
  height: 6px;
  background-color: #ffffff;
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(255, 255, 255, 0.8);
}

.hero-title {
  font-size: clamp(2.4rem, 5.2vw, 4.2rem);
  font-weight: 800;
  line-height: 1.14;
  letter-spacing: -1.8px;
  margin-bottom: 24px;
  color: #ffffff;
}

.gradient-text {
  background: linear-gradient(180deg, #ffffff 0%, #cbd5e1 55%, #64748b 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.hero-desc {
  max-width: 580px;
  margin: 0 auto 36px;
  font-size: 1.15rem;
  color: var(--text-secondary);
  line-height: 1.7;
  font-weight: 400;
}

.code-pill {
  font-family: var(--font-mono);
  font-size: 0.9em;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid var(--border-subtle);
  padding: 2px 7px;
  border-radius: 5px;
  color: var(--text-primary);
}

.code-pill.green {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
  border-color: rgba(16, 185, 129, 0.25);
}

.hero-cta-group {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  margin-bottom: 0;
  flex-wrap: wrap;
}

.btn-hero-primary {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 30px;
  background: #ffffff;
  color: #000000;
  border-radius: 10px;
  text-decoration: none;
  font-weight: 700;
  box-shadow: 0 4px 25px rgba(255, 255, 255, 0.18);
  transition: all 0.2s ease;
}

.btn-hero-primary:hover {
  transform: translateY(-2px);
  background: #f8fafc;
  box-shadow: 0 6px 30px rgba(255, 255, 255, 0.3);
}

.btn-text-block {
  display: flex;
  flex-direction: column;
  text-align: left;
}

.btn-main-text {
  font-size: 1rem;
  font-weight: 700;
}

.btn-sub-text {
  font-size: 0.72rem;
  color: #475569;
  font-weight: 500;
}

.btn-hero-secondary {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 26px;
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  color: var(--text-primary);
  border-radius: 10px;
  text-decoration: none;
  font-weight: 600;
  transition: all 0.2s ease;
}

.btn-hero-secondary:hover {
  background: var(--bg-card-hover);
  border-color: var(--border-strong);
  transform: translateY(-2px);
}"####
}
