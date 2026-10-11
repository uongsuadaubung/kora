//! Section headers styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Section Headers
   ========================================================================== */
.section {
  padding: 110px 0;
  position: relative;
}

.section-header {
  text-align: center;
  max-width: 720px;
  margin: 0 auto 56px;
}

.section-chip {
  display: inline-block;
  font-family: var(--font-mono);
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 1.5px;
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-subtle);
  padding: 4px 14px;
  border-radius: 99px;
  margin-bottom: 18px;
}

.section-title {
  font-size: clamp(2rem, 3.4vw, 2.7rem);
  font-weight: 800;
  letter-spacing: -1.2px;
  line-height: 1.2;
  margin-bottom: 16px;
  color: #ffffff;
}

.section-subtitle {
  color: var(--text-secondary);
  font-size: 1.05rem;
  line-height: 1.6;
}"####
}
