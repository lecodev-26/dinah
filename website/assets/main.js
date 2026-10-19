/* DINAH — hand-crafted interactions. No frameworks, no build step. */
(function () {
  'use strict';

  var prefersReduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  /* ---------- in-memory preferences (sandbox-safe) ----------
     Theme and language default to system settings on each visit. */
  var store = {
    get: function () { return null; },
    set: function () { /* no-op */ }
  };

  /* ---------- i18n ---------- */
  var ES = {
    skip: 'Saltar al contenido',
    nav: { modules: 'Módulos', architecture: 'Arquitectura', benchmarks: 'Benchmarks', limits: 'Límites', roadmap: 'Roadmap', blog: 'Blog' },
    heroTitle: 'El runtime que <em>muestra sus cuentas</em>.',
    heroLede: 'DINAH es una fundación Rust sin dependencias para aplicaciones deterministas — runtime, memoria, compute, ECS, simulación, networking y storage, construidos con contratos, límites probados y evidencia medida.',
    ctaCrates: 'Ver en crates.io',
    ctaGithub: 'Código en GitHub',
    statTests: 'tests en verde, incl. doc-tests',
    statDeps: 'dependencias en runtime',
    statUnsafe: 'bloques unsafe en el crate',
    statCoverage: 'cobertura de líneas exigida en CI',
    statPlatforms: 'plataformas con benchmarks en CI',
    statChecklist: 'puntos de checklist, 279 cerrados',
    modulesTitle: 'Ocho módulos, un contrato cada uno.',
    modulesLede: 'Cada módulo tiene una única responsabilidad y publica un contrato explícito y documentado. Sin schedulers ocultos, sin paralelismo implícito, sin asignaciones sorpresa.',
    modCore: 'Tipos y errores compartidos. Sin política ni kernels — el vocabulario en el que todos los módulos coinciden.',
    modRuntime: 'Scheduler acotado por llamada y sincronización con <code>Shared<T></code>. El trabajo se envía explícitamente y se espera antes de volver.',
    modMemory: '<code>Buffer<T></code> acotado y <code>SoaBuffer</code> de dos columnas. Reservas falibles, sin crecimiento implícito, sin unsafe.',
    modCompute: '<code>for_each</code>, <code>map</code> y <code>try_reduce</code> secuenciales, lotes explícitos, iteración paralela opt-in y pipelines componibles — secuencial por defecto, paralelo solo si lo pides.',
    modEcs: 'Handles generacionales de <code>Entity</code>, <code>World</code> single-owner, storage sparse-set tipado. Las queries de componentes y los sistemas paralelos llegan en v0.2.',
    modSimulation: 'Reloj de paso fijo, contrato de paso determinista, tests de replay y cinco modelos acotados — de física ASCII a flocking.',
    modNetworking: 'Wrappers TCP bloqueantes, predicción y reconciliación cliente, simulador determinista por ticks. El codec y el framing del wire son trabajo de v0.2.',
    modStorage: 'Journal append-only con detección de corrupción CRC-32: la corrupción completa falla, la escritura final interrumpida se recupera.',
    modCli: 'Proyectos basados en manifiesto: init, run, task, doctor y benchmark — con salida JSON para scripts y ejecución de tareas opt-in.',
    archTitle: 'Explícito por construcción.',
    archNote1H: 'Sin scheduler oculto',
    archNote1P: 'Los workers de Runtime son scoped y se espera su finalización antes de que cualquier llamada vuelva. Nada corre en segundo plano por importar un módulo.',
    archNote2H: 'El paralelismo es opt-in',
    archNote2P: 'Lo secuencial es la ruta por defecto. La iteración paralela existe, está medida y está documentada como más lenta para kernels ligeros.',
    archNote3H: 'Seguridad sin salidas de emergencia',
    archNote3P: 'Todo el crate prohíbe código unsafe. El ABI de C expone solo consultas escalares de ancho fijo — sin punteros, buffers ni callbacks.',
    benchTitle: 'Medido, no prometido.',
    benchLede: 'Medianas reales de las ejecuciones de benchmarks en CI, archivadas por commit con metadatos de entorno. Microbenchmarks en máquinas concretas — nunca números de marketing.',
    bench1H: 'Extensión por lotes SoA',
    bench1W: 'ns por fila insertada — bucle ingenuo vs. lote validado, Linux x86-64',
    bench1Batch: '64 filas · lote',
    bench1Naive2: '1.024 filas · ingenuo',
    bench1Batch2: '1.024 filas · lote',
    bench1Note: 'La validación por lote comprueba todo el lote una vez en lugar de por fila. Datos brutos: artifacts/benchmarks/memory-soa-batch-2026-10-11.txt',
    bench2H: 'Scheduler determinista',
    bench2W: 'scheduler vs. pool persistente de workers, relativo — la paridad es el objetivo de diseño',
    bench2Noop: 'ciclo de vida no-op',
    bench2Cpu: 'CPU-intenso en régimen',
    bench2Note: 'El despacho por llamada cuesta más que un pool persistente en tareas no-op — medido y documentado en vez de ocultado. Los kernels paralelos ligeros siguen secuenciales a propósito: 2 workers midieron 4–10× más lento que el bucle escalar con 262k elementos.',
    bench3H: 'Paso de simulación',
    bench3W: 'mediana ns por avance de reloj, cinco plataformas de CI',
    bench3Note: 'Construcción incluida; no es el coste aislado por tick en régimen. Siete suites en total: runtime, memory, compute, ECS, simulation, networking, storage.',
    limitsTitle: 'Lo que DINAH no es.',
    limitsLede: 'La mayoría de readmes listan funcionalidades. Este también lista las que faltan — los mismos límites honestos que encontrarás en la documentación y la revisión de seguridad.',
    limit1H: 'No es un motor de física o CFD',
    limit1P: 'Los modelos de simulación son puntos de partida educativos acotados — límites explícitos, sin afirmaciones de propósito general.',
    limit2H: 'Sin SIMD',
    limit2P: 'Los candidatos SIMD se investigaron y midieron; ninguno justificaba la complejidad para un kernel de producción todavía. El escalar sigue siendo la implementación de referencia.',
    limit3H: 'Networking sin codec de wire',
    limit3P: 'Existe TCP bloqueante; el framing, el encoder/decoder v1, auth y TLS están diseñados y programados para v0.2.',
    limit4H: 'Queries de ECS pendientes',
    limit4P: 'El acceso individual a componentes y el storage son sólidos; las queries combinadas y los sistemas paralelos seguros son la prioridad de v0.2.',
    limit5H: 'Todavía sin fuzzing',
    limit5P: 'Las suites de corrupción y robustez usan un corpus adversarial determinista — una primera barrera explícita, no fuzzing guiado por cobertura.',
    limit6H: 'Pre-alfa, semver 0.x',
    limit6P: 'La superficie de API v0.1 está congelada, pero 0.x permite cambios rompientes. Adóptalo con eso en mente.',
    roadmapTitle: 'Hacia dónde va.',
    roadmapLede: 'El diseño v0.2 está aprobado y registrado en el repositorio — las prioridades salen de brechas medidas en el contrato v0.1 congelado, no de la envidia de features.',
    tl1H: 'v0.1.0 — la fundación honesta',
    tl1P: 'API pública congelada, checklist de 287 puntos cerrada (279 completos, 8 aplazados por diseño), benchmarks en CI en cinco plataformas, 90,9 % de cobertura de líneas exigida, publicado en crates.io.',
    tl2H: 'v0.2 — cerrar las brechas reales',
    tl2A: 'Formato de wire de networking: framing v1, encoder/decoder y eventos estructurados versionados sobre TCP',
    tl2B: 'Queries de componentes ECS y ejecución paralela segura de sistemas, guiada por declaraciones de acceso validadas',
    tl2C: 'Transacciones en Storage y control de concurrencia para el journal',
    tl2D: 'Parser TOML completo de manifiestos y helpers opcionales de ciclo de vida caller-owned',
    blogTitle: 'Escritura, no marketing.',
    post1Read: '4 min de lectura',
    post1Title: 'Anunciando dinah 0.1.0',
    post1Excerpt: 'Cero dependencias, cero bloques unsafe, 293 tests y una regla: toda afirmación está medida, probada o marcada como no implementada. La historia de un release que se tomó en serio la honestidad.',
    post1Cta: 'Leer el anuncio →',
    footerBlurb: 'Un runtime modular Rust-first y fundación de compute de alto rendimiento. Documentación artesanal, evidencia medida, sin afirmaciones inverificables.',
    footerProject: 'Proyecto',
    footerDocs: 'Documentación',
    footerCommunity: 'Comunidad',
    footerNote: 'página artesanal — sin frameworks, sin build step',
    toastCopied: 'Copiado al portapapeles',
    themeDark: 'Cambiar a modo oscuro',
    themeLight: 'Cambiar a modo claro',
    post: {
      title: 'Anunciando dinah 0.1.0',
      readtime: '4 min de lectura',
      p1: 'Hoy dinah 0.1.0 está en crates.io. Es un runtime modular Rust-first y una fundación de compute de alto rendimiento — el tipo de frase que necesita desarrollo, así que desenvulémosla como lo haría el propio proyecto: con datos concretos.',
      p2: 'DINAH son ocho módulos — core, runtime, memory, compute, ecs, simulation, networking y storage — más una CLI basada en manifiestos. No tiene dependencias en runtime. Contiene cero bloques unsafe, impuesto en la raíz del crate con forbid(unsafe_code). Publica 293 tests en verde incluidos doc-tests, 90,9 % de cobertura de líneas exigida en CI, benchmarks archivados de cinco plataformas y un ABI de C público limitado a consultas escalares de ancho fijo.',
      h1: 'La regla que lo moldeó todo',
      p3: 'Al principio del proyecto se adoptó una única regla y decidió casi todo el diseño que vino después:',
      quote: 'Toda afirmación de la documentación está <em>medida</em>, <em>probada</em> o marcada explícitamente como <em>sin implementar todavía</em>.',
      p4: 'Suena obvio hasta que ves qué excluye. Sin afirmaciones de rendimiento sin un benchmark reproducible y archivado. Sin funcionalidades "próximamente" presentadas como capacidades. Sin esconder una limitación conocida con la esperanza de que nadie la pise. El README contiene literalmente una tabla de lo que DINAH no es.',
      p5: 'La consecuencia más visible: las rutas de compute paralelas están documentadas como más lentas para kernels ligeros. Lo son. Las medimos — con 262.144 elementos, dos workers corrieron entre 4 y 10,3 veces más lento que el bucle secuencial en los cinco runners de CI. Ese número está en la documentación, junto a la explicación de por qué lo secuencial es el defecto y el paralelismo es opt-in.',
      h2: 'Un checklist auditable',
      p6: 'El release lo impulsó un checklist de 287 puntos que vive en el repositorio. Cada punto enlaza su evidencia: tests, ejecuciones de benchmarks con metadatos de entorno, notas de la revisión de seguridad. El ciclo v0.1 cerró con 279 puntos completos, cero en progreso y 8 aplazados deliberadamente — cada punto aplazado no es una omisión silenciosa sino un objetivo v0.2 con su propio ID.',
      p7: 'La revisión de seguridad es un buen ejemplo del tono. Documenta aquello contra lo que el parser de manifiestos no protege, que los tests de corrupción son un corpus determinista y no fuzzing guiado por cobertura, y que todo el documento es una revisión estática, no una auditoría externa. El software pre-alfa que describe honestamente su propia superficie de ataque es raro, y es de lo que más orgulloso estoy.',
      h3: 'Pruébalo',
      p8: 'El MSRV es Rust 1.95, verificado empíricamente — el workspace falla de verdad al compilar en 1.94. Edición 2021, MIT OR Apache-2.0, y el paquete se verificó con cargo package y un dry-run de publicación antes del envío.',
      h4: 'Qué viene ahora',
      p9: 'El diseño v0.2 ya está escrito y aprobado. Las prioridades salen de brechas medidas en el contrato v0.1 congelado: el formato de wire de networking (framing v1, encoder/decoder y eventos estructurados versionados), las queries de componentes ECS y la ejecución paralela segura de sistemas, las transacciones de storage y un parser TOML completo de manifiestos. Sin fechas — solo diseño, evidencia y un checklist, que es exactamente como se construyó también este release.',
      p10: 'Si la infraestructura determinista y honesta es lo tuyo, el repositorio tiene guías de cada módulo, una línea base de rendimiento, una revisión de seguridad y el rastro de evidencia completo. Las críticas son bienvenidas — especialmente sobre lo que hicimos mal.',
      back: '← Volver al sitio',
      repo: 'Repositorio'
    }
  };

  var lang = store.get('dinah-lang') || 'en';
  var i18nApplied = false;

  function applyLang(l) {
    lang = l;
    document.documentElement.lang = l;
    document.querySelectorAll('[data-lang]').forEach(function (b) {
      b.setAttribute('aria-pressed', String(b.dataset.lang === l));
    });
    if (l === 'es') {
      document.querySelectorAll('[data-i18n]').forEach(function (el) {
        var key = el.getAttribute('data-i18n');
        var val = key.split('.').reduce(function (o, k) { return o && o[k]; }, ES);
        if (val !== undefined) el.textContent = val;
      });
      document.querySelectorAll('[data-i18n-html]').forEach(function (el) {
        var val = el.getAttribute('data-i18n-html').split('.').reduce(function (o, k) { return o && o[k]; }, ES);
        if (val !== undefined) el.innerHTML = val;
      });
      document.querySelectorAll('[data-i18n-nav]').forEach(function (el) {
        var val = ES.nav[el.getAttribute('data-i18n-nav')];
        if (val !== undefined) el.textContent = val;
      });
      i18nApplied = true;
    } else if (i18nApplied) {
      /* returning to English: reload the original document */
      window.location.reload();
      return;
    }
  }

  document.querySelectorAll('.lang-toggle button').forEach(function (btn) {
    btn.addEventListener('click', function () {
      if (btn.dataset.lang !== lang) {
        store.set('dinah-lang', btn.dataset.lang);
        applyLang(btn.dataset.lang);
      }
    });
  });

  /* ---------- theme ---------- */
  var theme = store.get('dinah-theme');
  if (!theme) theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';

  var SUN = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>';
  var MOON = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>';

  function setTheme(t, announce) {
    theme = t;
    document.documentElement.setAttribute('data-theme', t);
    store.set('dinah-theme', t);
    var btn = document.getElementById('themeToggle');
    if (btn) {
      btn.innerHTML = t === 'dark' ? SUN : MOON;
      btn.setAttribute('aria-label', t === 'dark' ? (ES.themeLight) : 'Switch to dark mode');
    }
    if (announce && window.__mountToaster && window.Sonner) window.__mountToaster(window.Sonner);
  }

  setTheme(theme, false);

  var themeBtn = document.getElementById('themeToggle');
  if (themeBtn) {
    themeBtn.addEventListener('click', function () {
      setTheme(theme === 'dark' ? 'light' : 'dark', true);
    });
  }

  /* ---------- sonner toaster mount (theme-synced) ---------- */
  window.__mountToaster = function (Sonner) {
    var el = document.getElementById('toaster');
    if (!el || !window.React || !window.ReactDOM) return;
    var root = ReactDOM.createRoot(el);
    root.render(React.createElement(Sonner.Toaster, {
      theme: theme,
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

  function toast(msg) {
    if (window.Sonner && window.Sonner.toast) { window.Sonner.toast.success(msg); return true; }
    return false;
  }

  /* ---------- scroll progress + header ---------- */
  var progress = document.getElementById('scrollProgress');
  var header = document.getElementById('siteHeader');
  var ticking = false;
  window.addEventListener('scroll', function () {
    if (ticking) return;
    ticking = true;
    requestAnimationFrame(function () {
      var h = document.documentElement;
      var max = h.scrollHeight - h.clientHeight;
      var p = max > 0 ? h.scrollTop / max : 0;
      if (progress) progress.style.width = (p * 100).toFixed(2) + '%';
      if (header) header.classList.toggle('is-scrolled', h.scrollTop > 8);
      ticking = false;
    });
  }, { passive: true });

  /* ---------- spotlight cards ---------- */
  if (!window.matchMedia('(hover: none)').matches) {
    document.querySelectorAll('.mod-card, .post-card, .bench-card').forEach(function (card) {
      card.addEventListener('pointermove', function (e) {
        var r = card.getBoundingClientRect();
        card.style.setProperty('--mx', (e.clientX - r.left) + 'px');
        card.style.setProperty('--my', (e.clientY - r.top) + 'px');
      });
    });
  }

  /* ---------- animated counters ---------- */
  function animateCount(el) {
    var target = parseFloat(el.dataset.count);
    var decimals = parseInt(el.dataset.decimals || '0', 10);
    if (prefersReduced) { el.textContent = target.toFixed(decimals); return; }
    var start = null;
    var dur = 900;
    function step(ts) {
      if (!start) start = ts;
      var t = Math.min((ts - start) / dur, 1);
      var eased = 1 - Math.pow(1 - t, 3);
      el.textContent = (target * eased).toFixed(decimals);
      if (t < 1) requestAnimationFrame(step);
    }
    requestAnimationFrame(step);
  }

  var counterObserver = new IntersectionObserver(function (entries) {
    entries.forEach(function (en) {
      if (en.isIntersecting) { animateCount(en.target); counterObserver.unobserve(en.target); }
    });
  }, { threshold: 0.6 });
  document.querySelectorAll('[data-count]').forEach(function (el) { counterObserver.observe(el); });

  /* ---------- benchmark bars ---------- */
  var barObserver = new IntersectionObserver(function (entries) {
    entries.forEach(function (en) {
      if (en.isIntersecting) {
        var w = en.target.dataset.width;
        requestAnimationFrame(function () { en.target.style.width = w + '%'; });
        barObserver.unobserve(en.target);
      }
    });
  }, { threshold: 0.4 });
  document.querySelectorAll('.bar-fill').forEach(function (el) { barObserver.observe(el); });

  /* ---------- terminal typing ---------- */
  var term = document.getElementById('terminalBody');
  if (term) {
    var lines = [
      { html: '<span class="prompt">$ </span><span class="cmd">cargo add dinah</span>' },
      { html: '<span class="out">    Updating crates.io index</span>' },
      { html: '<span class="out">      Adding dinah v0.1.0 to dependencies</span>' },
      { html: '<span class="ok">Resolved</span> <span class="out">· 0 deps · 0 unsafe</span>' }
    ];
    if (prefersReduced) {
      term.innerHTML = lines.map(function (l) { return '<div>' + l.html + '</div>'; }).join('');
    } else {
      term.innerHTML = '';
      var li = 0;
      function typeLine() {
        if (li >= lines.length) return;
        var div = document.createElement('div');
        var caret = document.createElement('span');
        caret.className = 'terminal-caret';
        /* move the caret: remove it from previous lines */
        var prev = term.querySelector('.terminal-caret');
        if (prev) prev.remove();
        div.innerHTML = lines[li].html;
        div.appendChild(caret);
        term.appendChild(div);
        /* simple fade-step per line, 420ms apart */
        div.style.opacity = '0';
        div.style.transition = 'opacity 260ms ' + 'ease-out';
        requestAnimationFrame(function () { div.style.opacity = '1'; });
        li++;
        setTimeout(typeLine, 420);
      }
      setTimeout(typeLine, 500);
    }
  }

  /* ---------- copy install command ---------- */
  var copyBtn = document.getElementById('copyCmd');
  if (copyBtn) {
    copyBtn.addEventListener('click', function () {
      var text = 'cargo add dinah';
      var done = function () {
        var ok = toast(lang === 'es' ? ES.toastCopied : 'Copied to clipboard');
        if (!ok) {
          copyBtn.innerHTML = '✓';
          setTimeout(function () { copyBtn.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg> copy'; }, 1600);
        }
      };
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, done);
      } else { done(); }
    });
  }

  /* ---------- mobile nav ---------- */
  var navBtn = document.getElementById('mobileNavBtn');
  var navPanel = document.getElementById('mobileNav');
  if (navBtn && navPanel) {
    navBtn.addEventListener('click', function () {
      var open = navPanel.classList.toggle('is-open');
      navBtn.setAttribute('aria-expanded', String(open));
    });
    navPanel.addEventListener('click', function (e) {
      if (e.target.tagName === 'A') {
        navPanel.classList.remove('is-open');
        navBtn.setAttribute('aria-expanded', 'false');
      }
    });
  }

  /* ---------- initial language ---------- */
  applyLang(lang);
})();
