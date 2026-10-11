//! Download CTA styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Download Section
   ========================================================================== */
.download-card {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.04) 0%, rgba(255, 255, 255, 0.01) 100%), var(--bg-card);
  border: 1px solid var(--border-strong);
  border-radius: 20px;
  padding: 64px 40px;
  text-align: center;
  box-shadow: 0 30px 100px rgba(0, 0, 0, 0.8);
}

.download-badge {
  font-family: var(--font-mono);
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 1.5px;
  color: var(--text-secondary);
  margin-bottom: 18px;
}

.download-title {
  font-size: clamp(2.2rem, 3.8vw, 3rem);
  font-weight: 800;
  letter-spacing: -1.2px;
  margin-bottom: 16px;
  color: #ffffff;
}

.download-desc {
  max-width: 620px;
  margin: 0 auto 40px;
  color: var(--text-secondary);
  font-size: 1.05rem;
}

.download-buttons {
  display: flex;
  justify-content: center;
  gap: 16px;
  margin-bottom: 36px;
  flex-wrap: wrap;
}

.btn-download-main {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 32px;
  background: #ffffff;
  color: #000000;
  border-radius: 10px;
  text-decoration: none;
  font-weight: 700;
  box-shadow: 0 4px 25px rgba(255, 255, 255, 0.18);
  transition: all 0.2s ease;
}

.btn-download-main:hover {
  transform: translateY(-2px);
  background: #f8fafc;
  box-shadow: 0 6px 30px rgba(255, 255, 255, 0.28);
}

.btn-download-sub {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 28px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-subtle);
  color: var(--text-primary);
  border-radius: 10px;
  text-decoration: none;
  font-weight: 600;
  transition: all 0.2s ease;
}

.btn-download-sub:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: var(--border-strong);
  transform: translateY(-2px);
}

.download-guarantee {
  font-size: 0.82rem;
  color: var(--text-muted);
}"####
}
