// What every page of the bot's site shares (SiteChrome): the menu bar on a phone, signing out, a word that signing in
// failed, the seat a player has somewhere - a table filling, a draft, a game - to get back to from any page, and the
// helpers the pages are written with (window.Cantrip). Served at /site.js; every link is relative to the site's root,
// which each page's <base> names.
(() => {
  "use strict";

  const esc = text => String(text ?? "").replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const plural = (n, one, many) => `${n} ${n === 1 ? one : (many ?? one + "s")}`;
  const page = () => document.body?.dataset.page ?? "";

  const FORMATS = {
    booster: "Booster draft", grid: "Grid draft", winston: "Winston draft", standard: "Standard Gauntlet", "quick-draft": "Quick draft",
  };
  const formatName = key => FORMATS[key] ?? key ?? "Draft";

  // To Discord and back to this page, signed in.
  function signIn(then) {
    location.assign(`signin?then=${encodeURIComponent(then ?? (page() || "home"))}`);
  }

  async function getJson(path) {
    const response = await fetch(path, { cache: "no-store", credentials: "same-origin" });
    if (!response.ok) throw new Error(`The server said ${response.status}.`);
    const body = await response.json();
    return body && "value" in body ? body.value : body;
  }

  // A POST of JSON, answered with the API's value; a 401 goes to sign in, and resolves to null.
  async function post(path, body) {
    let response;
    try {
      response = await fetch(path, {
        method: "POST", credentials: "same-origin", cache: "no-store",
        headers: { "Content-Type": "application/json" }, body: JSON.stringify(body ?? {}),
      });
    } catch {
      throw new Error("The server could not be reached. Check your connection and try again.");
    }
    if (response.status === 401) { signIn(); return null; }
    if (response.status === 204) return {};
    if (!response.ok) throw new Error(`Something went wrong (the server said ${response.status}). Try again.`);
    const text = await response.text();
    if (!text) return {};
    const value = JSON.parse(text);
    return value && typeof value === "object" && "value" in value ? value.value : value;
  }

  // Goes to a seat in this tab - the link the API answers with, its ticket after the # - or returns why not.
  async function seat(path, body) {
    try {
      const answer = await post(path, body);
      if (answer === null) return null;
      if (answer?.link) { location.assign(answer.link); return null; }
      return answer?.refusal ?? "No seat could be had.";
    } catch (error) {
      return error.message;
    }
  }

  // A short word at the foot of the window, gone after a while; errors stay until dismissed.
  function toast(text, kind) {
    let host = document.querySelector(".site-toasts");
    if (!host) {
      host = document.createElement("div");
      host.className = "site-toasts";
      host.setAttribute("aria-live", "polite");
      document.body.append(host);
    }
    const item = document.createElement("div");
    item.className = `site-toast${kind === "error" ? " error" : ""}`;
    item.setAttribute("role", kind === "error" ? "alert" : "status");
    item.innerHTML = `<span class="grow">${esc(text)}</span><button type="button" class="button quiet small">Dismiss</button>`;
    item.querySelector("button").addEventListener("click", () => item.remove());
    host.append(item);
    if (kind !== "error") setTimeout(() => item.remove(), 6000);
  }

  // Asks before something that cannot be taken back. Resolves true for the confirming answer, false otherwise.
  function confirm({ title, body, confirmLabel = "Confirm", cancelLabel = "Cancel", danger = false }) {
    return new Promise(resolve => {
      const dialog = document.createElement("dialog");
      dialog.setAttribute("aria-labelledby", "site-confirm-title");
      dialog.innerHTML = `<div class="dialog-body">
          <h2 id="site-confirm-title">${esc(title)}</h2>
          ${(Array.isArray(body) ? body : [body]).filter(Boolean).map(line => `<p>${esc(line)}</p>`).join("")}
          <div class="actions">
            <button type="button" class="button" data-answer="no">${esc(cancelLabel)}</button>
            <button type="button" class="button ${danger ? "danger solid" : "primary"}" data-answer="yes">${esc(confirmLabel)}</button>
          </div>
        </div>`;
      let answered = false;
      const done = yes => {
        if (answered) return;
        answered = true;
        dialog.close();
        dialog.remove();
        resolve(yes);
      };
      dialog.addEventListener("click", event => {
        const answer = event.target.closest("[data-answer]")?.dataset.answer;
        if (answer) done(answer === "yes");
        else if (event.target === dialog) done(false);
      });
      dialog.addEventListener("cancel", event => { event.preventDefault(); done(false); });
      document.body.append(dialog);
      dialog.showModal();
      // The safe answer has the focus, so Enter never confirms by accident.
      dialog.querySelector('[data-answer="no"]').focus();
    });
  }

  // --- The menu bar ---------------------------------------------------------------------------------------------

  function menu() {
    const header = document.querySelector(".site-header");
    const toggle = document.querySelector("[data-site-menu]");
    if (!header || !toggle) return;
    const set = open => {
      header.classList.toggle("open", open);
      toggle.setAttribute("aria-expanded", String(open));
    };
    toggle.addEventListener("click", () => set(!header.classList.contains("open")));
    document.addEventListener("keydown", event => { if (event.key === "Escape") set(false); });
  }

  async function signOut() {
    try { await fetch("signout", { method: "POST", credentials: "same-origin", headers: { "Content-Type": "application/json" }, body: "{}" }); }
    catch { /* signed out or not, the next page says */ }
    location.assign("./");
  }

  // --- The seat a player has somewhere ---------------------------------------------------------------------------

  let board = null;

  // A league run that wants the player somewhere: a match found, a draft under way, or a match being looked for.
  function leagueBanner(b) {
    const league = page => `leagues/${encodeURIComponent(page.id)}`;
    for (const l of b.leagues ?? []) {
      const mine = l.mine;
      if (mine?.opponent) {
        return { text: `Your <strong>${esc(l.name)}</strong> match against <strong>${esc(mine.opponent)}</strong> ${mine.matchReady ? "is ready" : "is about to begin"}.`,
          label: "Go to my match", action: () => seat("leagues/api/play", { league: l.id }) };
      }
    }
    for (const l of b.leagues ?? []) {
      const mine = l.mine;
      if (mine?.current?.stage === "Drafting") {
        return { text: `Your <strong>${esc(l.name)}</strong> draft is under way.`, label: "Back to my draft", action: () => seat("leagues/api/play", { league: l.id }) };
      }
      if (mine?.searchingSince) {
        return { text: `Looking for a <strong>${esc(l.name)}</strong> match&hellip; keep this site open.`, label: "Open the league", action: () => { location.assign(league(l)); return null; } };
      }
    }
    return null;
  }

  function bannerFor(b) {
    if (!b?.me) return null;
    const inLeague = leagueBanner(b);
    if (inLeague) return inLeague;
    const d = b.draft;
    if (d?.drafting) return { text: d.format ? `Your <strong>${esc(formatName(d.format))}</strong> is under way.` : "Your draft is under way.", label: "Back to my draft", action: () => seat("draft/api/return") };
    if (d?.table) {
      const seated = d.players && d.seats ? ` (${d.players.length} of ${d.seats} seated)` : "";
      return { text: `You have a seat at a <strong>${esc(formatName(d.format))}</strong> table that is filling${esc(seated)}.`, label: "Open my table", action: () => seat("draft/api/join", { table: d.table }) };
    }
    const g = b.constructed;
    if (g) {
      const what = g.stage === "Open" ? "waiting for an opponent" : g.stage === "Seated" ? "about to begin" : "being played";
      return { text: `Your <strong>${esc(formatName(g.format))}</strong> game is ${what}.`, label: "Open my game", action: () => seat("constructed/api/join", { game: g.id }) };
    }
    return null;
  }

  function renderBanner() {
    const banner = document.getElementById("site-banner");
    if (!banner) return;
    // The player's own page shows all of it as cards; a page can say it shows the seat itself.
    const shown = page() === "home" || document.body.dataset.ownBanner === "true" ? null : bannerFor(board);
    if (!shown) { banner.hidden = true; banner.innerHTML = ""; return; }
    banner.innerHTML = `<div class="site-banner-inner"><span class="grow">${shown.text}</span><button type="button" class="button primary small" data-site-banner>${esc(shown.label)}</button></div>`;
    banner.hidden = false;
    banner.querySelector("[data-site-banner]").addEventListener("click", async () => {
      const refusal = await shown.action();
      if (refusal) toast(refusal, "error");
    });
  }

  async function watch() {
    if (!document.querySelector(".site-me")) return;
    const look = async () => {
      try { board = await getJson("home/api/board?brief=1"); } catch { return; }
      renderBanner();
      document.dispatchEvent(new CustomEvent("cantrip:board", { detail: board }));
    };
    await look();
    setInterval(look, 5000);
  }

  window.Cantrip = { esc, plural, post, seat, getJson, toast, confirm, signIn, formatName, get board() { return board; } };

  function start() {
    menu();
    document.addEventListener("click", event => {
      if (event.target.closest("[data-site-signout]")) { event.preventDefault(); signOut(); }
      else if (event.target.closest("[data-site-signin]")) { event.preventDefault(); signIn(); }
    });
    const query = new URLSearchParams(location.search);
    if (query.get("signin") === "failed") {
      toast("Signing in with Discord didn't go through. Try again.", "error");
      query.delete("signin");
      history.replaceState(null, "", location.pathname + (query.size ? `?${query}` : "") + location.hash);
    }
    watch();
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
  else start();
})();
