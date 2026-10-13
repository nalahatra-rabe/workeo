# 🎯 Workeo — Trouve ton job avec un système multi-agents IA

**Workeo** est une application de recherche d'emploi et d'alternance qui analyse ton CV,
explore le web, filtre les offres et t'explique pourquoi chaque poste te correspond.

Backend **Rust** (Axum + SQLx + Tokio) pour la performance, et interface **React +
TypeScript** (Vite, Tailwind CSS, shadcn/ui, icônes Lucide) servie par le backend.

---

## ✨ Fonctionnalités

- 🔐 **Comptes utilisateurs** — inscription / connexion sécurisées (Argon2id + JWT).
- 🧑‍💼 **Multi-profils** — chaque compte gère plusieurs profils, avec des données,
  recherches et candidatures **clairement séparées** par profil.
- 📄 **Upload de CV** (PDF, DOCX, TXT) avec **extraction de texte** et **analyse IA**.
- 🧠 **Catégorisation du profil** — compétences, séniorité, domaines, postes cibles, mots-clés.
- 🌐 **Deep search / web scraping** — interroge plusieurs sources **en parallèle**
  (Arbeitnow, Remotive, RemoteOK + source démo hors-ligne).
- 🏷️ **Agent de catégorisation des offres** — nettoie, déduplique, détecte catégorie,
  compétences, contrat, télétravail et salaire.
- ⭐ **Score de compatibilité** (0-100) avec **explications concrètes** pour chaque offre.
- 📬 **Suivi de candidatures** (kanban : sauvegardées → postulées → entretiens → offres).
- 🎨 **UI moderne** — palette plate bleu & blanc, icônes Lucide, onboarding animé.
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
- [Node.js](https://nodejs.org) 20+ et [pnpm](https://pnpm.io) (pour compiler l'interface)

### Lancer

```bash
cp .env.example .env
cargo run --release
```

Ouvrez **http://localhost:8080** 🎉

L'interface précompilée est déjà présente dans `web/`, il n'y a donc rien à builder
pour un simple essai. Pour la reconstruire après une modification :

```bash
cd frontend
pnpm install
pnpm build      # génère ../web, servi par Axum
```

Pour développer l'interface avec rechargement à chaud (proxy `/api` → `:8080`) :

```bash
cd frontend && pnpm dev   # http://localhost:5173
```

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

1. **Créer un compte** (nom, email, mot de passe).
2. **Onboarding animé** → création du premier profil.
3. **Uploader le CV** → l'IA extrait compétences, expériences, formation.
4. **Tableau de bord** → catégories, mots-clés, recommandations.
5. **Recherche** → les agents scannent le web en parallèle, classent et expliquent.
6. **Postuler** → suivi dans le kanban candidatures.

---

## 🔌 API REST

Authentification par **JWT** : `Authorization: Bearer <token>`.

| Méthode | Route | Description |
|---------|-------|-------------|
| `POST` | `/api/auth/register` | Créer un compte |
| `POST` | `/api/auth/login` | Se connecter (renvoie un JWT) |
| `GET`  | `/api/auth/me` | Utilisateur courant |
| `GET`  | `/api/health` | État du service + IA |
| `GET`  | `/api/stats?profile_id=` | Statistiques (scopées au profil) |
| `GET`  | `/api/profiles` | Lister mes profils |
| `POST` | `/api/profiles` | Créer un profil |
| `GET`  | `/api/profiles/:id` | Lire un profil |
| `PUT`  | `/api/profiles/:id` | Mettre à jour |
| `DELETE` | `/api/profiles/:id` | Supprimer un profil |
| `POST` | `/api/profiles/:id/default` | Définir comme profil par défaut |
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
│   ├── auth.rs             # Argon2 + JWT + extracteur AuthUser
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
│   └── routes/              # endpoints REST (auth, profiles, search…)
├── frontend/                # 💻 sources de l'interface (Vite + React + TS)
│   ├── src/
│   │   ├── components/ui/       # composants shadcn/ui
│   │   ├── contexts/            # auth & profils (état global)
│   │   ├── pages/               # login, onboarding, dashboard, recherche…
│   │   └── lib/                 # client API, types, utilitaires
│   └── vite.config.ts           # build → ../web
├── web/                     # interface compilée, servie par Axum
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
