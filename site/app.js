(() => {
  const workbookTabs = Array.from(document.querySelectorAll('.workbook-tab'));
  const img = document.getElementById('workbook-image');
  const titleEl = document.getElementById('workbook-title');
  const descEl = document.getElementById('workbook-desc');

  function selectWorkbookTab(tab) {
    if (!tab || !img) return;
    workbookTabs.forEach((btn) => btn.setAttribute('aria-selected', btn === tab ? 'true' : 'false'));
    const nextSrc = tab.dataset.src;
    if (nextSrc && img.getAttribute('src') !== nextSrc) {
      img.classList.add('is-fading');
      const loader = new Image();
      loader.onload = () => {
        img.src = nextSrc;
        img.alt = tab.dataset.alt || '';
        img.classList.remove('is-fading');
      };
      loader.onerror = () => {
        img.src = nextSrc;
        img.classList.remove('is-fading');
      };
      loader.src = nextSrc;
    }
    if (titleEl) titleEl.textContent = tab.dataset.title || '';
    if (descEl) descEl.textContent = tab.dataset.desc || '';
  }

  workbookTabs.forEach((tab, index) => {
    tab.addEventListener('click', () => selectWorkbookTab(tab));
    tab.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
        e.preventDefault();
        const dir = e.key === 'ArrowRight' ? 1 : -1;
        const next = workbookTabs[(index + dir + workbookTabs.length) % workbookTabs.length];
        next.focus();
        selectWorkbookTab(next);
      }
    });
  });

  const osTabs = Array.from(document.querySelectorAll('.os-tab'));
  const osPanels = Array.from(document.querySelectorAll('.os-panel'));

  function selectOs(os) {
    osTabs.forEach((tab) => tab.setAttribute('aria-selected', tab.dataset.os === os ? 'true' : 'false'));
    osPanels.forEach((panel) => { panel.hidden = panel.id !== 'panel-' + os; });
  }

  osTabs.forEach((tab, index) => {
    tab.addEventListener('click', () => selectOs(tab.dataset.os));
    tab.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
        e.preventDefault();
        const dir = e.key === 'ArrowRight' ? 1 : -1;
        const next = osTabs[(index + dir + osTabs.length) % osTabs.length];
        next.focus();
        selectOs(next.dataset.os);
      }
    });
  });

  const ua = (navigator.userAgent || '').toLowerCase();
  if (ua.includes('win')) selectOs('windows');
  else if (ua.includes('linux') && !ua.includes('android')) selectOs('linux');
  else selectOs('macos');

  document.querySelectorAll('[data-copy-target]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const target = document.getElementById(btn.dataset.copyTarget);
      if (!target) return;
      const text = target.textContent.trim();
      try {
        await navigator.clipboard.writeText(text);
        btn.textContent = 'Copied';
        btn.setAttribute('data-copied', 'true');
        window.setTimeout(() => {
          btn.textContent = 'Copy';
          btn.removeAttribute('data-copied');
        }, 1800);
      } catch {
        const range = document.createRange();
        range.selectNodeContents(target);
        const sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(range);
      }
    });
  });
})();
