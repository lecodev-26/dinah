/* DINAH — sonner toaster mount (ES modules via import map). */
import * as React from 'react';
import { createRoot } from 'react-dom/client';
import * as Sonner from 'sonner';

window.React = React;
window.ReactDOM = { createRoot };
window.Sonner = Sonner;

window.__mountToaster = function () {
  var el = document.getElementById('toaster');
  if (!el) return;
  el.hidden = false;
  if (!window.__toasterRoot) window.__toasterRoot = createRoot(el);
  window.__toasterRoot.render(React.createElement(Sonner.Toaster, {
    theme: document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light',
    position: 'bottom-right',
    richColors: false,
    toastOptions: {
      style: {
        background: 'var(--color-surface)',
        border: '1px solid var(--color-border)',
        color: 'var(--color-text)',
        fontFamily: "'Satoshi', sans-serif",
        borderRadius: '0.625rem'
      }
    }
  }));
};

window.__mountToaster();
