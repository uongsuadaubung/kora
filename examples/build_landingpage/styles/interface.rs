//! Interface gallery & philosophy styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   App Interface Showcase Gallery
   ========================================================================== */
.ui-gallery-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 24px;
}

.ui-card {
  background: var(--bg-card);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  border-radius: 16px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: all 0.3s ease;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
}

.ui-card:hover {
  transform: translateY(-4px);
  border-color: var(--border-strong);
  box-shadow: 0 20px 45px rgba(0, 0, 0, 0.6), 0 0 30px rgba(255, 255, 255, 0.04);
}

.ui-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid var(--border-subtle);
}

.ui-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #38bdf8;
  box-shadow: 0 0 8px rgba(56, 189, 248, 0.5);
}

.ui-window-title {
  font-family: var(--font-mono);
  font-size: 0.72rem;
  color: var(--text-secondary);
  font-weight: 600;
  letter-spacing: 0.3px;
}

.ui-img-frame {
  background: #050607;
  padding: 14px 14px 0 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid var(--border-subtle);
  overflow: hidden;
}

.ui-img-frame img {
  width: 100%;
  height: auto;
  border-radius: 8px 8px 0 0;
  display: block;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6);
  transition: transform 0.3s ease;
}

.ui-card:hover .ui-img-frame img {
  transform: scale(1.02);
}

.ui-card-body {
  padding: 22px 20px;
  display: flex;
  flex-direction: column;
  flex-grow: 1;
}

.ui-tab-tag {
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.1);
  border: 1px solid rgba(56, 189, 248, 0.25);
  padding: 3px 8px;
  border-radius: 4px;
  align-self: flex-start;
  margin-bottom: 10px;
  letter-spacing: 0.5px;
}

.ui-card-title {
  font-size: 1.08rem;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 12px;
}

.ui-card-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ui-card-list li {
  font-size: 0.84rem;
  color: var(--text-secondary);
  line-height: 1.5;
  position: relative;
  padding-left: 14px;
}

.ui-card-list li::before {
  content: '•';
  color: #38bdf8;
  position: absolute;
  left: 0;
  font-size: 0.9rem;
}

.ui-card-list strong {
  color: #ffffff;
}

.ui-card-list code {
  font-family: var(--font-mono);
  font-size: 0.78rem;
  background: rgba(255, 255, 255, 0.08);
  padding: 1px 5px;
  border-radius: 4px;
  color: #38bdf8;
}

/* ==========================================================================
   Design Philosophy Callout
   ========================================================================== */
.philosophy-box {
  margin-top: 48px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.035) 0%, rgba(255, 255, 255, 0.01) 100%), var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: 16px;
  padding: 36px 32px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.4);
}

.philosophy-header {
  margin-bottom: 28px;
  text-align: left;
}

.philosophy-badge {
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 1.5px;
  color: #38bdf8;
  display: inline-block;
  padding: 4px 12px;
  background: rgba(56, 189, 248, 0.08);
  border: 1px solid rgba(56, 189, 248, 0.25);
  border-radius: 99px;
  margin-bottom: 12px;
}

.philosophy-title {
  font-size: clamp(1.25rem, 2.2vw, 1.6rem);
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 8px;
  letter-spacing: -0.5px;
}

.philosophy-desc {
  color: var(--text-secondary);
  font-size: 0.92rem;
  line-height: 1.6;
  max-width: 820px;
}

.philosophy-desc strong {
  color: #ffffff;
}

.philosophy-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.philosophy-item {
  background: rgba(0, 0, 0, 0.45);
  border: 1px solid var(--border-subtle);
  border-radius: 12px;
  padding: 22px 20px;
  display: flex;
  flex-direction: column;
  transition: all 0.25s ease;
}

.philosophy-item:hover {
  border-color: rgba(255, 255, 255, 0.18);
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.philosophy-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: rgba(56, 189, 248, 0.1);
  border: 1px solid rgba(56, 189, 248, 0.25);
  color: #38bdf8;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
  flex-shrink: 0;
}

.philosophy-content h4 {
  font-size: 0.98rem;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 8px;
  line-height: 1.4;
}

.philosophy-content p {
  font-size: 0.84rem;
  color: var(--text-secondary);
  line-height: 1.55;
  margin: 0;
}

.philosophy-content strong {
  color: #ffffff;
}

.philosophy-content em {
  font-style: normal;
  color: #cbd5e1;
}"####
}
