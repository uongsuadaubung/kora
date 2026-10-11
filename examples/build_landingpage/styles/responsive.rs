//! Responsive media query breakpoints

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Responsive Breakpoints
   ========================================================================== */
@media (max-width: 992px) {
  .telemetry-grid, .specs-grid, .benchmark-cards-grid, .ui-gallery-grid, .philosophy-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .guide-steps-grid {
    grid-template-columns: 1fr;
  }

  .demo-display {
    grid-template-columns: 1fr;
  }

  .comparison-divider {
    flex-direction: row;
    height: auto;
  }

  .divider-line {
    width: 100%;
    height: 1px;
  }
}

@media (max-width: 768px) {
  .nav-links {
    display: none;
  }

  .telemetry-grid, .specs-grid, .benchmark-cards-grid, .ui-gallery-grid, .philosophy-grid {
    grid-template-columns: 1fr;
  }

  .benchmark-showcase, .philosophy-box {
    padding: 24px 18px;
  }

  .soundpack-guide-card {
    padding: 24px 18px;
  }

  .faq-grid {
    grid-template-columns: 1fr;
  }

  .faq-card {
    padding: 20px 18px;
  }

  .faq-a {
    padding-left: 0;
    margin-top: 6px;
  }

  .accordion-summary {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
    padding: 16px 18px;
  }

  .naming-categories {
    grid-template-columns: 1fr;
  }

  .naming-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
  }

  .row-file {
    text-align: left;
  }

  .comparison-table {
    display: block;
    overflow-x: auto;
  }

  .macro-table-box {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .macro-table {
    min-width: 650px;
  }

  .macro-table-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .warning-grid {
    grid-template-columns: 1fr;
  }

  .hero {
    padding: 70px 0 40px;
  }

  .download-card {
    padding: 40px 20px;
  }
}"####
}
