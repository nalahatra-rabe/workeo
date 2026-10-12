/* ═══════════════════════════════════════════════════════════
   Workeo — Application front-end
   ═══════════════════════════════════════════════════════════ */

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

const State = {
  profile: null,
  aiMode: "heuristique",
  lastResults: [],
  sort: "score",
};

// ─── API ────────────────────────────────────────────────────
const api = {
  async req(method, url, body, isForm = false) {
    const opts = { method, headers: {} };
    if (body && !isForm) {
      opts.headers["Content-Type"] = "application/json";
      opts.body = JSON.stringify(body);
    } else if (body && isForm) {
      opts.body = body;
    }
    const res = await fetch(url, opts);
    const text = await res.text();
    let data = null;
    try { data = text ? JSON.parse(text) : null; } catch { data = text; }
    if (!res.ok) {
      throw new Error((data && data.error) || `Erreur ${res.status}`);
    }
    return data;
  },
  get(u) { return this.req("GET", u); },
  post(u, b) { return this.req("POST", u, b); },
  put(u, b) { return this.req("PUT", u, b); },
  patch(u, b) { return this.req("PATCH", u, b); },
  del(u) { return this.req("DELETE", u); },
};

// ─── Toasts ─────────────────────────────────────────────────
function toast(msg, type = "", ms = 4200) {
  const el = document.createElement("div");
  el.className = `toast ${type}`;
  el.innerHTML = msg;
  $("#toasts").appendChild(el);
  setTimeout(() => { el.style.opacity = "0"; setTimeout(() => el.remove(), 300); }, ms);
}

// ─── Navigation ─────────────────────────────────────────────
const VIEW_META = {
  dashboard: ["Tableau de bord", "Votre assistant de recherche d'emploi"],
  search: ["Recherche", "Lancez les agents IA sur le web"],
  jobs: ["Offres", "Toutes les offres collectées"],
  applications: ["Candidatures", "Suivez vos postulations"],
  profile: ["Mon profil", "Informations et CV"],
};

function navigate(view) {
  $$(".nav-item[data-view]").forEach((b) => b.classList.toggle("active", b.dataset.view === view));
  $$(".view").forEach((v) => v.classList.toggle("active", v.id === `view-${view}`));
  const meta = VIEW_META[view] || ["", ""];
  $("#viewTitle").textContent = meta[0];
  $("#viewSub").textContent = meta[1];
  if (view === "applications") loadApplications();
  if (view === "jobs") loadExplore();
  if (view === "profile") renderAnalysis($("#analysisResult"), State.profile, true);
}

// ─── Init ───────────────────────────────────────────────────
async function init() {
  $$(".nav-item[data-view]").forEach((b) =>
    b.addEventListener("click", () => navigate(b.dataset.view))
  );
  $("#themeToggle").addEventListener("click", toggleTheme);
  $("#btnStart").addEventListener("click", () => openOnboarding());
  $("#btnQuickSearch").addEventListener("click", () => navigate("search"));
  $("#btnGeoloc").addEventListener("click", locateMe);
  $("#btnCreateProfile").addEventListener("click", createProfile);
  $("#btnSaveProfile").addEventListener("click", saveProfile);
  $("#btnReanalyze").addEventListener("click", reanalyze);
  $("#btnSearch").addEventListener("click", runSearch);
  $("#btnExplore").addEventListener("click", loadExplore);
  $("#btnRefreshRecos").addEventListener("click", loadRecommendations);
  $("#sortResults").addEventListener("change", (e) => { State.sort = e.target.value; renderResults(); });
  $("#exploreQuery").addEventListener("keydown", (e) => e.key === "Enter" && loadExplore());
  $("#exploreCategory").addEventListener("change", loadExplore);
  $("#jobModal").addEventListener("click", (e) => { if (e.target.id === "jobModal") closeJobModal(); });
  $("#onboarding").addEventListener("click", (e) => { if (e.target.id === "onboarding" && State.profile) closeOnboarding(); });
  document.addEventListener("keydown", (e) => { if (e.key === "Escape") { closeJobModal(); } });

  setupDropzone();
  applySavedTheme();
  await loadHealth();
  await loadSources();
  await loadCategories();

  const savedId = localStorage.getItem("workeo_profile_id");
  if (savedId) {
    try {
      State.profile = await api.get(`/api/profiles/${savedId}`);
      onProfileReady(false);
    } catch {
      localStorage.removeItem("workeo_profile_id");
      openOnboarding();
    }
  } else {
    openOnboarding();
  }
}

// ─── Santé / IA ─────────────────────────────────────────────
async function loadHealth() {
  try {
    const h = await api.get("/api/health");
    State.aiMode = h.llm.mode;
    const on = h.llm.mode === "llm";
    $("#aiDot").className = `dot ${on ? "on" : "off"}`;
    $("#aiText").innerHTML = on
      ? `IA <b>${h.llm.model}</b> (${h.llm.provider})`
      : `Moteur <b>heuristique</b> (IA hors-ligne)`;
    if (!on && h.llm.enabled) {
      toast("⚠️ IA configurée mais injoignable — utilisation du moteur heuristique local.", "", 6000);
    }
  } catch {
    $("#aiText").textContent = "Serveur indisponible";
  }
}

async function loadSources() {
  try {
    const s = await api.get("/api/sources");
    const sel = $("#searchSources");
    sel.innerHTML = "";
    s.sources.forEach((src) => {
      const opt = document.createElement("option");
      opt.value = src.name;
      opt.textContent = src.label;
      opt.selected = src.enabled;
      sel.appendChild(opt);
    });
  } catch {}
}

async function loadCategories() {
  try {
    const cats = await api.get("/api/categories");
    const sel = $("#exploreCategory");
    cats.forEach((c) => {
      const o = document.createElement("option");
      o.value = c; o.textContent = c;
      sel.appendChild(o);
    });
  } catch {}
}

// ─── Onboarding / profil ────────────────────────────────────
function openOnboarding() { $("#onboarding").classList.add("open"); }
function closeOnboarding() { $("#onboarding").classList.remove("open"); }

async function createProfile() {
  const name = $("#obName").value.trim();
  if (!name) { toast("Le nom est requis", "error"); return; }
  const btn = $("#btnCreateProfile");
  btn.disabled = true; btn.innerHTML = `<span class="spinner"></span> Création…`;
  try {
    const contract = $("#obContract").value;
    State.profile = await api.post("/api/profiles", {
      full_name: name,
      email: $("#obEmail").value.trim() || null,
      location: $("#obLocation").value.trim() || null,
      headline: $("#obHeadline").value.trim() || null,
      remote_preference: $("#obRemote").value,
      contract_types: contract ? [contract] : [],
    });
    localStorage.setItem("workeo_profile_id", State.profile.id);
    closeOnboarding();
    onProfileReady(true);
    toast("✅ Profil créé ! Ajoutez votre CV pour l'analyse IA.", "success");
  } catch (e) {
    toast(`Erreur : ${e.message}`, "error");
  } finally {
    btn.disabled = false; btn.textContent = "Créer mon profil →";
  }
}

function onProfileReady(isNew) {
  $("#dashboardEmpty").style.display = "none";
  $("#dashboardFull").style.display = "block";
  fillProfileForm();
  $("#searchQuery").value = State.profile?.target_roles?.[0] || State.profile?.headline || "";
  $("#searchLocation").value = State.profile?.location || "";
  loadStats();
  renderProfilePanels();
  loadRecommendations();
  if (isNew) navigate("profile");
}

function fillProfileForm() {
  const p = State.profile; if (!p) return;
  $("#pName").value = p.full_name || "";
  $("#pEmail").value = p.email || "";
  $("#pPhone").value = p.phone || "";
  $("#pLocation").value = p.location || "";
  $("#pHeadline").value = p.headline || "";
  $("#pRemote").value = p.remote_preference || "any";
  $("#pSalary").value = p.salary_min || "";
  renderCvInfo();
}

async function saveProfile() {
  if (!State.profile) return;
  const btn = $("#btnSaveProfile");
  btn.disabled = true; btn.innerHTML = `<span class="spinner"></span> Enregistrement…`;
  try {
    State.profile = await api.put(`/api/profiles/${State.profile.id}`, {
      full_name: $("#pName").value.trim(),
      email: $("#pEmail").value.trim(),
      phone: $("#pPhone").value.trim(),
      location: $("#pLocation").value.trim(),
      headline: $("#pHeadline").value.trim(),
      remote_preference: $("#pRemote").value,
      salary_min: $("#pSalary").value ? parseInt($("#pSalary").value, 10) : null,
    });
    toast("✅ Profil enregistré", "success");
    loadStats();
  } catch (e) { toast(`Erreur : ${e.message}`, "error"); }
  finally { btn.disabled = false; btn.textContent = "💾 Enregistrer"; }
}

// ─── CV upload ──────────────────────────────────────────────
function setupDropzone() {
  const dz = $("#dropzone");
  const input = $("#cvInput");
  dz.addEventListener("click", () => input.click());
  input.addEventListener("change", () => input.files[0] && uploadCv(input.files[0]));
  ["dragenter", "dragover"].forEach((ev) =>
    dz.addEventListener(ev, (e) => { e.preventDefault(); dz.classList.add("drag"); })
  );
  ["dragleave", "drop"].forEach((ev) =>
    dz.addEventListener(ev, (e) => { e.preventDefault(); dz.classList.remove("drag"); })
  );
  dz.addEventListener("drop", (e) => {
    const f = e.dataTransfer.files[0];
    if (f) uploadCv(f);
  });
}

async function uploadCv(file) {
  if (!State.profile) { toast("Créez d'abord un profil", "error"); return; }
  const dz = $("#dropzone");
  dz.innerHTML = `<div class="big"><span class="spinner dark" style="width:34px;height:34px;border-width:3px"></span></div>
                  <p><strong>Analyse de ${file.name}…</strong></p>
                  <p class="hint">Les agents IA extraient vos compétences, expériences et catégorie.</p>`;
  const form = new FormData();
  form.append("file", file);
  try {
    State.profile = await api.req("POST", `/api/profiles/${State.profile.id}/cv`, form, true);
    toast("✅ CV analysé par l'IA !", "success");
    resetDropzone();
    renderCvInfo();
    renderProfilePanels();
    fillProfileForm();
    renderAnalysis($("#analysisResult"), State.profile, true);
    loadStats();
    loadRecommendations();
    navigate("dashboard");
  } catch (e) {
    toast(`Échec de l'analyse : ${e.message}`, "error", 6000);
    resetDropzone();
  }
}

function resetDropzone() {
  const dz = $("#dropzone");
  dz.innerHTML = `<div class="big">📎</div>
    <p><strong>Glissez votre CV ici</strong> ou cliquez</p>
    <p class="hint">PDF, DOCX ou TXT — analyse automatique par l'IA</p>
    <input type="file" id="cvInput" accept=".pdf,.docx,.doc,.txt,.md" style="display:none" />`;
  setupDropzone();
}

function renderCvInfo() {
  const p = State.profile;
  const el = $("#cvInfo");
  if (!p?.cv_text) { el.innerHTML = ""; return; }
  const words = p.cv_text.split(/\s+/).length;
  el.innerHTML = `<div class="reason" style="color:var(--text-dim)">
    <span>📄 <b>${escapeHtml(p.cv_filename || "CV")}</b> — ${words} mots analysés</span></div>`;
}

async function reanalyze() {
  if (!State.profile) return;
  const btn = $("#btnReanalyze");
  btn.disabled = true; btn.innerHTML = `<span class="spinner dark"></span> Analyse IA…`;
  try {
    State.profile = await api.post(`/api/profiles/${State.profile.id}/analyze`);
    renderProfilePanels();
    fillProfileForm();
    renderAnalysis($("#analysisResult"), State.profile, true);
    toast("🧠 Profil ré-analysé", "success");
    loadRecommendations();
  } catch (e) { toast(`Erreur : ${e.message}`, "error"); }
  finally { btn.disabled = false; btn.textContent = "🧠 Ré-analyser avec l'IA"; }
}

// ─── Dashboard rendering ────────────────────────────────────
async function loadStats() {
  try {
    const s = await api.get("/api/stats");
    $("#statsGrid").innerHTML = `
      ${statCard("💼", s.jobs, "Offres collectées")}
      ${statCard("🌍", s.remote_jobs, "Offres télétravail")}
      ${statCard("🏷️", (s.by_category || []).length, "Catégories métier")}
      ${statCard("👥", s.profiles, "Profils")}`;
  } catch {}
}

function statCard(ico, n, l) {
  return `<div class="card"><div class="stat"><div class="ico">${ico}</div>
    <div class="n">${n}</div><div class="l">${l}</div></div></div>`;
}

function renderProfilePanels() {
  renderSummary($("#profileSummary"));
  renderCategories($("#profileCategories"));
}

function renderSummary(el) {
  const p = State.profile;
  if (!p) { el.innerHTML = `<div class="empty">Aucun profil</div>`; return; }
  const exp = p.experiences || [];
  let html = "";
  html += row("Titre", p.headline || "—");
  html += row("Séniorité", p.seniority || "—");
  if (p.years_experience) html += row("Expérience", `${p.years_experience} ans`);
  html += row("Catégorie", p.category || "—");
  if (p.summary) html += `<p style="color:var(--text-dim);font-size:13.5px;margin:12px 0">${escapeHtml(p.summary)}</p>`;
  if (p.skills?.length) {
    html += `<div style="margin-top:12px"><div class="l" style="font-size:12.5px;color:var(--text-dim);margin-bottom:7px">Compétences détectées (${p.skills.length})</div>
      <div class="chips">${p.skills.slice(0, 24).map((s) => `<span class="chip primary">${escapeHtml(s)}</span>`).join("")}</div></div>`;
  }
  if (exp.length) {
    html += `<div style="margin-top:14px"><div style="font-size:12.5px;color:var(--text-dim);margin-bottom:7px">Expériences</div>`;
    exp.slice(0, 4).forEach((e) => {
      html += `<div class="reason" style="color:var(--text)"><span>${escapeHtml(e.title || "Poste")} ${e.company ? "· " + escapeHtml(e.company) : ""} ${e.start ? `<span style="color:var(--text-faint)">(${e.start}${e.end ? "–" + e.end : ""})</span>` : ""}</span></div>`;
    });
    html += `</div>`;
  }
  el.innerHTML = html;
}

function renderCategories(el) {
  const p = State.profile;
  if (!p) { el.innerHTML = ""; return; }
  let html = "";
  if (p.category) {
    html += `<div class="chip primary" style="font-size:14px;padding:7px 14px">🎯 ${escapeHtml(p.category)}</div>`;
  }
  if (p.domains?.length) {
    html += `<div style="margin-top:14px"><div style="font-size:12.5px;color:var(--text-dim);margin-bottom:7px">Domaines</div>
      <div class="chips">${p.domains.map((d) => `<span class="chip">${escapeHtml(d)}</span>`).join("")}</div></div>`;
  }
  if (p.target_roles?.length) {
    html += `<div style="margin-top:14px"><div style="font-size:12.5px;color:var(--text-dim);margin-bottom:7px">Postes suggérés</div>
      <div class="chips">${p.target_roles.map((r) => `<span class="chip success">${escapeHtml(r)}</span>`).join("")}</div></div>`;
  }
  if (p.search_keywords?.length) {
    html += `<div style="margin-top:14px"><div style="font-size:12.5px;color:var(--text-dim);margin-bottom:7px">Mots-clés de recherche</div>
      <div class="chips">${p.search_keywords.map((k) => `<span class="chip warning">${escapeHtml(k)}</span>`).join("")}</div></div>`;
  }
  el.innerHTML = html || `<div class="empty">Analysez un CV pour détecter vos catégories</div>`;
}

function row(label, value) {
  return `<div style="display:flex;justify-content:space-between;gap:12px;padding:6px 0;border-bottom:1px solid var(--border)">
    <span style="color:var(--text-dim);font-size:13px">${label}</span>
    <span style="font-weight:600;font-size:13.5px;text-align:right">${escapeHtml(String(value))}</span></div>`;
}

function renderAnalysis(el, p, full) {
  if (!p) { el.innerHTML = `<div class="empty"><div class="big">📄</div>Ajoutez un CV pour lancer l'analyse.</div>`; return; }
  if (!p.cv_text) { el.innerHTML = `<div class="empty"><div class="big">📄</div>Aucun CV analysé pour l'instant.</div>`; return; }
  let html = `<div class="grid cols-3">`;
  html += analysisBlock("🛠️ Compétences", p.skills);
  html += analysisBlock("🌐 Langues", (p.languages || []).map((l) => l.name));
  html += analysisBlock("🎓 Formation", (p.education || []).map((e) => e.degree).filter(Boolean));
  html += `</div>`;
  el.innerHTML = html;
}

function analysisBlock(title, items) {
  if (!items || !items.length) return `<div><div style="font-size:12.5px;color:var(--text-dim);margin-bottom:8px">${title}</div><span style="color:var(--text-faint);font-size:13px">—</span></div>`;
  return `<div><div style="font-size:12.5px;color:var(--text-dim);margin-bottom:8px">${title} (${items.length})</div>
    <div class="chips">${items.slice(0, 30).map((s) => `<span class="chip">${escapeHtml(String(s))}</span>`).join("")}</div></div>`;
}

// ─── Recommandations ────────────────────────────────────────
async function loadRecommendations() {
  if (!State.profile) return;
  const list = $("#recosList");
  list.innerHTML = skeletons(3);
  try {
    const recos = await api.get(`/api/profiles/${State.profile.id}/recommendations`);
    if (!recos.length) {
      list.innerHTML = `<div class="empty"><div class="big">🔍</div>Pas encore de recommandations.<br>Lancez une recherche pour alimenter l'IA.</div>`;
      return;
    }
    list.innerHTML = recos.slice(0, 8).map((j) => jobCardHtml(j, true)).join("");
    bindJobCards(list, recos);
  } catch (e) { list.innerHTML = `<div class="empty">Erreur : ${escapeHtml(e.message)}</div>`; }
}

// ─── Recherche ──────────────────────────────────────────────
async function runSearch() {
  if (!State.profile) { toast("Créez d'abord un profil", "error"); return; }
  const query = $("#searchQuery").value.trim();
  const location = $("#searchLocation").value.trim();
  const contract = $("#searchContract").value;
  const remoteOnly = $("#searchRemote").value === "true";
  const sources = [...$("#searchSources").selectedOptions].map((o) => o.value);

  const btn = $("#btnSearch");
  btn.disabled = true; btn.innerHTML = `<span class="spinner"></span> Recherche en cours…`;

  const progress = $("#searchProgress");
  const steps = $("#agentSteps");
  progress.style.display = "block";
  $("#searchResults").style.display = "none";
  renderAgentSteps(steps, 0);

  const stepTimers = animateSteps(steps, [
    "Dérivation de la requête depuis votre profil",
    "Scraping multi-sources en parallèle",
    "Catégorisation & nettoyage des offres",
    "Calcul des scores de compatibilité",
    "Classement et persistance des résultats",
  ]);

  try {
    const res = await api.post("/api/search", {
      profile_id: State.profile.id,
      query: query || null,
      location: location || null,
      remote_only: remoteOnly,
      contract_type: contract || null,
      sources: sources.length ? sources : null,
    });
    stepTimers.forEach(clearInterval);
    renderAgentSteps(steps, 5, true);

    State.lastResults = res.results;
    $("#searchResults").style.display = "block";
    $("#resultsCount").textContent = `${res.total} offre(s) classées`;
    const errs = res.source_errors?.length ? ` · ${res.source_errors.join(", ")}` : "";
    $("#resultsMeta").textContent = `via ${res.run.sources.join(", ")} · ${res.run.jobs_new} nouvelles${errs}`;

    renderResults();

    if (res.total === 0) {
      $("#resultsList").innerHTML = `<div class="empty"><div class="big">🤷</div>Aucune offre trouvée. Essayez d'autres mots-clés ou d'élargir la zone géographique.</div>`;
    }
    if (Notification && Notification.permission === "granted") {
      new Notification("Workeo", { body: `${res.total} offres trouvées pour vous !` });
    }
    setTimeout(() => { progress.style.display = "none"; }, 1200);
  } catch (e) {
    stepTimers.forEach(clearInterval);
    toast(`Recherche échouée : ${e.message}`, "error", 6000);
    progress.style.display = "none";
  } finally {
    btn.disabled = false; btn.textContent = "🚀 Lancer la recherche (agents IA)";
  }
}

function renderAgentSteps(el, upTo, allDone = false) {
  const labels = [
    "Dérivation de la requête depuis votre profil",
    "Scraping multi-sources en parallèle",
    "Catégorisation & nettoyage des offres",
    "Calcul des scores de compatibilité",
    "Classement et persistance des résultats",
  ];
  el.innerHTML = labels.map((l, i) => {
    const done = allDone || i < upTo;
    return `<div class="agent-step ${done ? "done" : ""}">
      <span class="n">${done ? "✓" : i + 1}</span><span>${l}</span></div>`;
  }).join("");
}

function animateSteps(el, labels) {
  let i = 0;
  const base = 700;
  const delays = [400, 1600, 1400, 900];
  const timers = [];
  const tick = () => {
    i++;
    renderAgentSteps(el, i);
    if (i < labels.length) {
      timers.push(setTimeout(tick, delays[i - 1] || base));
    }
  };
  timers.push(setTimeout(tick, base));
  return timers;
}

function renderResults() {
  const list = $("#resultsList");
  let results = [...State.lastResults];
  if (State.sort === "recent") {
    results.sort((a, b) => (b.posted_at || b.scraped_at || "").localeCompare(a.posted_at || a.scraped_at || ""));
  } else if (State.sort === "salary") {
    results.sort((a, b) => (b.salary_min || 0) - (a.salary_min || 0));
  } else {
    results.sort((a, b) => b.score - a.score);
  }
  list.innerHTML = results.map((j) => jobCardHtml(j, true)).join("");
  bindJobCards(list, results);
}

// ─── Cartes d'offres ────────────────────────────────────────
function jobCardHtml(job, withScore) {
  const score = job.score ?? 0;
  const ring = withScore && score > 0 ? scoreRing(score) : "";
  const salary = job.salary_min || job.salary_max
    ? `<span class="chip success">💰 ${fmtSalary(job)}</span>` : "";
  const chips = [];
  if (job.remote) chips.push(`<span class="chip primary">🌍 Télétravail</span>`);
  if (job.contract_type) chips.push(`<span class="chip">📝 ${escapeHtml(job.contract_type)}</span>`);
  if (job.seniority) chips.push(`<span class="chip">📈 ${escapeHtml(job.seniority)}</span>`);
  if (job.category) chips.push(`<span class="chip">🏷️ ${escapeHtml(job.category)}</span>`);
  chips.push(`<span class="chip">🔗 ${escapeHtml(job.source)}</span>`);

  const reasons = (job.reasons || []).slice(0, 3)
    .map((r) => `<div class="reason"><span>${escapeHtml(r)}</span></div>`).join("");

  return `<article class="job-card" data-job-id="${job.id}">
    <div class="job-body">
      <h3>${escapeHtml(job.title)}</h3>
      <div class="company">🏢 ${escapeHtml(job.company)}${job.location ? " · 📍 " + escapeHtml(job.location) : ""}</div>
      <div class="meta">${chips.join("")}${salary}</div>
      ${reasons ? `<div class="reasons">${reasons}</div>` : ""}
    </div>
    ${ring}
  </article>`;
}

function scoreRing(score) {
  const color = score >= 85 ? "var(--success)" : score >= 70 ? "var(--primary)" : score >= 50 ? "var(--accent)" : "var(--text-faint)";
  return `<div class="score-ring" style="background:conic-gradient(${color} ${score * 3.6}deg, var(--panel-2) 0);">
    <div style="width:46px;height:46px;border-radius:50%;background:var(--panel);display:grid;place-items:center;flex-direction:column;line-height:1">
      ${score}<small>%</small>
    </div></div>`;
}

function bindJobCards(root, jobs) {
  $$(".job-card", root).forEach((card) => {
    card.addEventListener("click", () => {
      const job = jobs.find((j) => j.id === card.dataset.jobId);
      if (job) openJobModal(job);
    });
  });
}

async function openJobModal(job) {
  const modal = $("#jobModal");
  const reasons = (job.reasons || []).map((r) => `<div class="reason"><span>${escapeHtml(r)}</span></div>`).join("");
  modal.innerHTML = `<div class="modal">
    <div style="display:flex;gap:16px;align-items:flex-start">
      <div style="flex:1">
        <h2 style="font-size:21px;margin-bottom:6px">${escapeHtml(job.title)}</h2>
        <div style="color:var(--text-dim);font-size:14px">🏢 ${escapeHtml(job.company)}${job.location ? " · 📍 " + escapeHtml(job.location) : ""}</div>
      </div>
      ${job.score ? scoreRing(job.score) : ""}
    </div>
    <div class="chips" style="margin-top:14px">
      ${job.remote ? `<span class="chip primary">🌍 Télétravail</span>` : ""}
      ${job.contract_type ? `<span class="chip">📝 ${escapeHtml(job.contract_type)}</span>` : ""}
      ${job.seniority ? `<span class="chip">📈 ${escapeHtml(job.seniority)}</span>` : ""}
      ${job.category ? `<span class="chip">🏷️ ${escapeHtml(job.category)}</span>` : ""}
      ${(job.salary_min || job.salary_max) ? `<span class="chip success">💰 ${fmtSalary(job)}</span>` : ""}
    </div>
    ${reasons ? `<div style="margin-top:18px"><div style="font-weight:700;font-size:14px;margin-bottom:8px">Pourquoi cette offre vous correspond</div>${reasons}</div>` : ""}
    ${job.skills?.length ? `<div style="margin-top:16px"><div style="font-size:12.5px;color:var(--text-dim);margin-bottom:8px">Compétences demandées</div><div class="chips">${job.skills.map((s) => `<span class="chip">${escapeHtml(s)}</span>`).join("")}</div></div>` : ""}
    ${job.summary ? `<p style="margin-top:16px;color:var(--text-dim);font-size:13.5px">${escapeHtml(job.summary)}</p>` : ""}
    ${job.description ? `<details style="margin-top:14px"><summary style="cursor:pointer;color:var(--primary-soft)">Description complète</summary><p style="margin-top:10px;color:var(--text-dim);font-size:13px;white-space:pre-wrap">${escapeHtml(job.description).slice(0, 4000)}</p></details>` : ""}
    <div style="display:flex;gap:10px;margin-top:22px;flex-wrap:wrap">
      <button class="btn primary" id="btnApply">📬 Postuler</button>
      <button class="btn" id="btnSaveApp">💾 Sauvegarder</button>
      ${job.url ? `<a class="btn" href="${escapeAttr(job.url)}" target="_blank" rel="noopener">🔗 Offre originale</a>` : ""}
      <button class="btn ghost" id="btnCloseModal" style="margin-left:auto">Fermer</button>
    </div>
  </div>`;

  modal.classList.add("open");
  $("#btnCloseModal").addEventListener("click", closeJobModal);
  $("#btnApply").addEventListener("click", () => applyToJob(job, "applied"));
  $("#btnSaveApp").addEventListener("click", () => applyToJob(job, "saved"));
}

function closeJobModal() { $("#jobModal").classList.remove("open"); }

async function applyToJob(job, status) {
  if (!State.profile) { toast("Créez un profil d'abord", "error"); return; }
  try {
    await api.post("/api/applications", { profile_id: State.profile.id, job_id: job.id, status });
    toast(status === "applied" ? "📬 Candidature enregistrée !" : "💾 Offre sauvegardée", "success");
    closeJobModal();
    if ($("#view-applications").classList.contains("active")) loadApplications();
  } catch (e) { toast(`Erreur : ${e.message}`, "error"); }
}

// ─── Candidatures ───────────────────────────────────────────
const STATUS_COLS = [
  ["saved", "💾 Sauvegardées"],
  ["applied", "📤 Postulées"],
  ["interview", "🗣️ Entretiens"],
  ["offer", "🎉 Offres"],
];

async function loadApplications() {
  if (!State.profile) return;
  const el = $("#kanban");
  el.innerHTML = `<div class="skeleton" style="height:120px"></div>`.repeat(4);
  try {
    const apps = await api.get(`/api/profiles/${State.profile.id}/applications`);
    el.innerHTML = STATUS_COLS.map(([status, label]) => {
      const items = apps.filter((a) => a.status === status);
      return `<div class="kanban-col">
        <h4>${label} <span>${items.length}</span></h4>
        ${items.map((a) => appMiniCard(a)).join("") || `<div style="color:var(--text-faint);font-size:12.5px;text-align:center;padding:14px 0">—</div>`}
      </div>`;
    }).join("");
  } catch (e) { el.innerHTML = `<div class="empty">Erreur : ${escapeHtml(e.message)}</div>`; }
}

function appMiniCard(a) {
  const job = a.job || {};
  const next = { saved: "applied", applied: "interview", interview: "offer", offer: null }[a.status];
  return `<div class="mini-card">
    <div class="t">${escapeHtml(job.title || "Offre")}</div>
    <div class="c">🏢 ${escapeHtml(job.company || "")}</div>
    <div style="display:flex;gap:6px;margin-top:9px;flex-wrap:wrap">
      ${next ? `<button class="btn sm" data-app="${a.id}" data-status="${next}">→ ${statusLabel(next)}</button>` : ""}
      <button class="btn sm ghost" data-del-app="${a.id}">🗑️</button>
    </div>
  </div>`;
}

function statusLabel(s) {
  return { applied: "Postulé", interview: "Entretien", offer: "Offre", saved: "Sauvé" }[s] || s;
}

document.addEventListener("click", async (e) => {
  const mv = e.target.closest("[data-app]");
  if (mv) {
    try { await api.patch(`/api/applications/${mv.dataset.app}`, { status: mv.dataset.status }); loadApplications(); toast("Statut mis à jour", "success"); }
    catch (err) { toast(err.message, "error"); }
  }
  const del = e.target.closest("[data-del-app]");
  if (del) {
    try { await api.del(`/api/applications/${del.dataset.delApp}/delete`); loadApplications(); }
    catch (err) { toast(err.message, "error"); }
  }
});

// ─── Explorateur d'offres ───────────────────────────────────
async function loadExplore() {
  const el = $("#exploreList");
  el.innerHTML = skeletons(4);
  const q = $("#exploreQuery").value.trim();
  const cat = $("#exploreCategory").value;
  const params = new URLSearchParams();
  if (q) params.set("q", q);
  if (cat) params.set("category", cat);
  params.set("limit", "50");
  try {
    const jobs = await api.get(`/api/jobs?${params}`);
    if (!jobs.length) { el.innerHTML = `<div class="empty"><div class="big">📭</div>Aucune offre. Lancez une recherche d'abord.</div>`; return; }
    el.innerHTML = jobs.map((j) => jobCardHtml(j, false)).join("");
    bindJobCards(el, jobs);
  } catch (e) { el.innerHTML = `<div class="empty">Erreur : ${escapeHtml(e.message)}</div>`; }
}

// ─── Géolocalisation ────────────────────────────────────────
function locateMe() {
  if (!navigator.geolocation) { toast("Géolocalisation non supportée", "error"); return; }
  toast("📍 Obtention de votre position…");
  navigator.geolocation.getCurrentPosition(
    async (pos) => {
      const { latitude, longitude } = pos.coords;
      try {
        const r = await fetch(`https://nominatim.openstreetmap.org/reverse?format=json&lat=${latitude}&lon=${longitude}&zoom=10`);
        const d = await r.json();
        const city = d.address?.city || d.address?.town || d.address?.village || d.address?.state || "";
        if (city) {
          $("#searchLocation").value = city;
          if (State.profile && !State.profile.location) {
            State.profile.location = city;
            $("#pLocation").value = city;
          }
          toast(`📍 Localisation détectée : ${city}`, "success");
        } else { toast("Position obtenue", "success"); }
      } catch { toast("Position obtenue mais ville introuvable", ""); }
    },
    () => toast("Géolocalisation refusée", "error")
  );
  if (window.Notification && Notification.permission === "default") Notification.requestPermission();
}

// ─── Thème ──────────────────────────────────────────────────
function toggleTheme() {
  const cur = document.documentElement.dataset.theme;
  const next = cur === "dark" ? "light" : "dark";
  document.documentElement.dataset.theme = next;
  localStorage.setItem("workeo_theme", next);
  $("#themeToggle").innerHTML = `<span class="ico">${next === "dark" ? "🌙" : "☀️"}</span> Thème`;
}
function applySavedTheme() {
  const t = localStorage.getItem("workeo_theme") || "dark";
  document.documentElement.dataset.theme = t;
  $("#themeToggle").innerHTML = `<span class="ico">${t === "dark" ? "🌙" : "☀️"}</span> Thème`;
}

// ─── Helpers ────────────────────────────────────────────────
function fmtSalary(job) {
  const cur = job.salary_currency === "USD" ? "$" : job.salary_currency === "GBP" ? "£" : "€";
  if (job.salary_min && job.salary_max && job.salary_min !== job.salary_max)
    return `${(job.salary_min / 1000).toFixed(0)}k–${(job.salary_max / 1000).toFixed(0)}k ${cur}`;
  const v = job.salary_min || job.salary_max;
  return v ? `${(v / 1000).toFixed(0)}k ${cur}` : "";
}

function skeletons(n) {
  return Array.from({ length: n }, () => `<div class="skeleton" style="height:92px;margin-bottom:12px"></div>`).join("");
}

function escapeHtml(s) {
  return String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}
function escapeAttr(s) { return escapeHtml(s); }

init();
