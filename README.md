# 🎯 Workeo — Trouve ton job avec un système multi-agents IA

**Workeo** est une application de recherche d'emploi et d'alternance qui analyse ton CV,
explore le web, filtre les offres et t'explique pourquoi chaque poste te correspond.

Écrite en **Rust** (Axum + SQLx + Tokio) pour la performance, avec une interface web
moderne servie directement par le backend — **aucun build front-end nécessaire**.

---

## ✨ Fonctionnalités

- 🧑‍💼 **Profil en 30 secondes** — infos minimales, le reste est déduit du CV.
- 📄 **Upload de CV** (PDF, DOCX, TXT) avec **extraction de texte** et **analyse IA**.
- 🧠 **Catégorisation du profil** — compétences, séniorité, domaines, postes cibles, mots-clés.
- 🌐 **Deep search / web scraping** — interroge plusieurs sources **en parallèle**
  (Arbeitnow, Remotive, RemoteOK + source démo hors-ligne).
- 🏷️ **Agent de catégorisation des offres** — nettoie, déduplique, détecte catégorie,
  compétences, contrat, télétravail et salaire.
- ⭐ **Score de compatibilité** (0-100) avec **explications concrètes** pour chaque offre.
- 📬 **Suivi de candidatures** (kanban : sauvegardées → postulées → entretiens → offres).
- 📍 **Géolocalisation navigateur**, notifications, thème clair/sombre, responsive.
- 🔌 **Fonctionne sans IA externe** : si aucun LLM n'est configuré, un moteur
  heuristique local prend le relais. Avec **Ollama** ou une **API compatible OpenAI**,
  les agents utilisent un vrai LLM.

## 🤖 Les agents

| Agent | Rôle |
|-------|------|
| `CvAnalyzerAgent` | Extrait les infos structurées du CV |
| `ProfileCategorizerAgent` | Catégorise le profil + génère mots-clés / titres cibles |
| `JobCategorizerAgent` | Nettoie et catégorise les offres issues du scraping |
| `MatcherAgent` | Score la compatibilité profil ↔ offres |
| `SearchOrchestrator` | Coordonne tout le pipeline |

---

## 🚀 Démarrage rapide

### Prérequis
- [Rust](https://rustup.rs) (stable)

### Lancer

```bash
cp .env.example .env
cargo run --release
```

Ouvrez **http://localhost:8080** 🎉

> Par défaut, l'app tente d'utiliser **Ollama**. Si Ollama n'est pas lancé, elle bascule
> automatiquement en **mode heuristique** (aucune configuration requise).
> Pour forcer le mode heuristique : `LLM_PROVIDER=none`.

### Activer une vraie IA (optionnel, gratuit, local)

```bash
# Installer Ollama : https://ollama.com
ollama pull llama3.1
ollama serve
```

Puis dans `.env` :

```env
LLM_PROVIDER=ollama
OLLAMA_MODEL=llama3.1
```

### Utiliser une API cloud (compatible OpenAI)

```env
LLM_PROVIDER=openai
OPENAI_API_KEY=sk-...
OPENAI_BASE_URL=https://api.openai.com/v1   # ou Groq, Together, LM Studio…
OPENAI_MODEL=gpt-4o-mini
```

---

## 🐳 Docker

```bash
docker build -t workeo .
docker run -p 8080:8080 -v workeo-data:/app/data workeo
```

---

## 🧭 Parcours utilisateur

1. **Accueil** → « Commencer ».
2. **Créer un profil** (nom + ville + objectif).
3. **Uploader le CV** → l'IA extrait compétences, expériences, formation.
4. **Tableau de bord** → catégories, mots-clés, recommandations.
5. **Recherche** → les agents scannent le web en parallèle, classent et expliquent.
6. **Postuler** → suivi dans le kanban candidatures.

---

## 🔌 API REST

| Méthode | Route | Description |
|---------|-------|-------------|
| `GET`  | `/api/health` | État du service + IA |
| `GET`  | `/api/stats` | Statistiques globales |
| `POST` | `/api/profiles` | Créer un profil |
| `GET`  | `/api/profiles/:id` | Lire un profil |
| `PUT`  | `/api/profiles/:id` | Mettre à jour |
| `POST` | `/api/profiles/:id/cv` | Upload CV (multipart) + analyse IA |
| `POST` | `/api/profiles/:id/analyze` | (Ré)analyser le profil |
| `GET`  | `/api/profiles/:id/recommendations` | Top offres recommandées |
| `GET`  | `/api/profiles/:id/applications` | Suivi des candidatures |
| `POST` | `/api/search` | Lancer une recherche multi-agents |
| `GET`  | `/api/jobs` | Explorer les offres |
| `GET`  | `/api/jobs/:id` | Détail d'une offre |
| `POST` | `/api/applications` | Créer une candidature |

---

## 🏗️ Architecture

```
Navigateur (SPA)  ──►  Axum (API + statique)
                          │
        ┌─────────────────┼──────────────────────┐
        ▼                 ▼                      ▼
   Agents IA         Scraper (parallèle)     SQLite (SQLx)
   orchestrés        Arbeitnow / Remotive    profils, offres,
                     RemoteOK / Démo        matchs, candidatures
        │
        ▼
   LLM (Ollama / OpenAI / heuristiques)
```

Détails complets dans [`SPEC.md`](./SPEC.md).

---

## 📁 Structure du projet

```
workeo/
├── src/
│   ├── main.rs              # point d'entrée + serveur Axum
│   ├── config.rs            # configuration (.env)
│   ├── models.rs            # types du domaine
│   ├── db.rs                # SQLite + schéma
│   ├── state.rs             # état partagé
│   ├── error.rs             # erreurs HTTP
│   ├── agents/              # 🤖 le système multi-agents
│   │   ├── llm.rs               # client LLM (Ollama/OpenAI/heuristique)
│   │   ├── heuristics.rs        # moteur local (dictionnaires, parsing)
│   │   ├── cv_analyzer.rs       # Agent 1 — analyse du CV
│   │   ├── profile_categorizer.rs # Agent 2 — catégorisation du profil
│   │   ├── job_categorizer.rs   # Agent 3 — catégorisation des offres
│   │   ├── matcher.rs           # Agent 4 — scoring de compatibilité
│   │   └── orchestrator.rs      # Agent 5 — orchestration
│   ├── scraper/             # 🌐 scraping web multi-sources
│   ├── services/            # extraction de texte CV (PDF/DOCX/TXT)
│   └── routes/              # endpoints REST
├── web/                     # interface (HTML/CSS/JS vanilla)
├── examples/sample_cv.txt   # CV d'exemple
├── SPEC.md                  # spécification produit & technique
└── Dockerfile
```

---

## 🧪 Essayer sans CV

Un CV d'exemple est fourni : [`examples/sample_cv.txt`](./examples/sample_cv.txt).
Créez un profil, puis glissez ce fichier dans la zone d'upload.

---

## 📜 Licence

MIT
