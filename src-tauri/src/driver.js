// Drives the site's own voice mode. The selectors arrive as JSON arguments, never as code.
(() => {
  if (window.top !== window || window.__onvox) return;
  const report = (r) => {
    try { window.__TAURI_INTERNALS__.invoke("page_report", { report: r }); } catch (e) {}
  };
  const visible = (el) => !!(el && el.offsetParent);
  // "text:Log in" matches a visible button or link whose text is exactly "Log in"; anything else is a CSS selector.
  const find = (list) => {
    for (const s of list) {
      if (s.startsWith("text:")) {
        const want = s.slice(5).trim();
        const el = [...document.querySelectorAll("button,a,[role=button]")]
          .find((b) => visible(b) && (b.innerText || "").trim() === want);
        if (el) return el;
      } else {
        const el = [...document.querySelectorAll(s)].find(visible);
        if (el) return el;
      }
    }
    return null;
  };
  // Only banners and dialogs: the AI's own words in the chat may mention a "limit".
  const limitShown = (texts) => {
    const shown = [...document.querySelectorAll("[role=alert],[role=dialog],[role=status],[aria-live]")]
      .map((e) => (e.innerText || "").toLowerCase()).join("\n");
    return texts.some((t) => shown.includes(t.toLowerCase()));
  };

  let timer = null;
  const watch = (fn) => { clearInterval(timer); timer = setInterval(fn, 500); };

  // Answers a cookie banner, preferring "reject non-essential": the list is ordered that way.
  const dismissConsent = (sel) => {
    const b = find(sel.consent_dismiss || []);
    if (b) b.click();
  };

  let loginAnswered = false;

  window.__onvox = {
    // Called repeatedly while the page loads; answers once, when either the voice button or a login sign shows.
    checkLogin(sel) {
      dismissConsent(sel);
      if (loginAnswered) return;
      if (find(sel.start_voice)) { loginAnswered = true; report({ kind: "logged_in" }); }
      else if (find(sel.logged_out)) { loginAnswered = true; report({ kind: "login_needed" }); }
    },
    start(sel) {
      // The page may still be loading: wait up to 8 s for the voice button before giving up.
      const until = Date.now() + 8000;
      const tryStart = () => {
        dismissConsent(sel);
        if (limitShown(sel.limit_texts)) {
          clearInterval(timer);
          return report({ kind: "failed", fault: "limit_reached" });
        }
        const button = find(sel.start_voice);
        if (button) return monitor(sel, button);
        if (Date.now() < until) return;
        clearInterval(timer);
        const fault = find(sel.logged_out) ? "not_logged_in" : "voice_button_missing";
        report({ kind: "failed", fault });
      };
      watch(tryStart);
      tryStart();
    },
    stop(sel) {
      // For 10 s keep closing voice mode, in case it comes up after the user already cancelled.
      const until = Date.now() + 10000;
      const close = () => {
        const b = find(sel.stop_voice);
        if (b) b.click();
        if (Date.now() > until) clearInterval(timer);
      };
      watch(close);
      close();
    },
  };

  // Clicks the voice button, then reports when voice mode comes up and when it ends.
  function monitor(sel, button) {
    button.click();
    let active = false;
    watch(() => {
      if (limitShown(sel.limit_texts)) {
        clearInterval(timer);
        return report({ kind: "failed", fault: "limit_reached" });
      }
      const now = !!find(sel.voice_active);
      if (now && !active) report({ kind: "voice_started" });
      if (!now && active) {
        clearInterval(timer);
        return report({ kind: "voice_ended" });
      }
      active = now;
    });
  }
})();
