//! FAQ section styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   FAQ Section
   ========================================================================== */
.faq-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  max-width: 1040px;
  margin: 0 auto;
}

.faq-card {
  background: var(--bg-card);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  border-radius: 14px;
  padding: 26px 24px;
  display: flex;
  flex-direction: column;
  transition: all 0.25s ease;
}

.faq-card:hover {
  border-color: rgba(255, 255, 255, 0.18);
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.faq-q {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 12px;
}

.faq-icon {
  font-size: 1.25rem;
  line-height: 1.2;
  flex-shrink: 0;
}

.faq-q h3 {
  font-size: 1.02rem;
  font-weight: 700;
  color: #ffffff;
  line-height: 1.45;
  margin: 0;
}

.faq-a {
  font-size: 0.88rem;
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 0;
  padding-left: 36px;
}

.faq-a strong {
  color: #ffffff;
}

.faq-a code {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  background: rgba(255, 255, 255, 0.08);
  padding: 2px 6px;
  border-radius: 4px;
  color: #38bdf8;
}"####
}
