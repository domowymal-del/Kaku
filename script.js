document.querySelectorAll('[data-copy]').forEach((button) => {
  button.addEventListener('click', async () => {
    const text = button.getAttribute('data-copy');
    const hint = button.querySelector('[data-copy-hint]');
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      if (hint) {
        const original = hint.textContent;
        hint.textContent = document.documentElement.lang === 'en' ? 'Copied' : '已复制';
        window.setTimeout(() => {
          hint.textContent = original;
        }, 1500);
      }
    } catch {
      if (hint) {
        hint.textContent = document.documentElement.lang === 'en' ? 'Copy failed' : '复制失败';
      }
    }
  });
});
