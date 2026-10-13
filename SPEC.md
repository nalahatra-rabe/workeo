# Workeo — Spécification produit & technique

> **Workeo** est une application de recherche d'emploi et d'alternance pilotée par un
> **système multi-agents**. Elle analyse le CV d'un utilisateur, catégorise son profil,
> scrape le web à la recherche d'offres, nettoie/catégorise ces offres, puis calcule un
> **score de compatibilité** et recommande les meilleurs postes.

Écrite en **Rust** (Axum + SQLx + Tokio) pour la performance, avec une interface web
réactive servie directement par le backend (zéro build front-end requis).

---

## 1. Vision produit

Deux publics principaux :

| Persona              | Besoin principal                                            |
|----------------------|-------------------------------------------------------------|
| **Personne "normale"** | Trouver un emploi rapidement sans connaître le jargon RH |
| **Étudiant**         | Trouver une **alternance / stage / premier emploi**          |

L'application doit être **compréhensible en 2 minutes** :
1. Je crée mon profil (infos + CV).
2. L'IA analyse et comprend mon profil.
3. L'IA cherche, filtre et m'explique pourquoi chaque offre me correspond.
4. Je postule et je suis mes candidatures.

---

## 2. Le flow utilisateur

```
┌─────────────┐    ┌──────────────┐    ┌────────────────┐    ┌──────────────┐
│  Onboarding │ →  │  Profil + CV │ →  │ Analyse IA CV  │ →  │  Catégorisation│
│  (0 friction)│    │  (upload)    │    │ (extraction)   │    │  du profil    │
└─────────────┘    └──────────────┘    └────────────────┘    └──────┬───────┘
                                                                     │
┌─────────────┐    ┌──────────────┐    ┌────────────────┐    ┌──────▼───────┐
│ Candidature │ ←  │ Recommandations│ ← │ Scoring/Match  │ ←  │  Recherche   │
│  + suivi    │    │  (meilleur job)│    │ (agents)       │    │ multi-sources│
└─────────────┘    └──────────────┘    └────────────────┘    └──────────────┘
                                                              (web scraping)
```

### Menus / écrans

1. **Accueil** — pitch + « Commencer » (géolocalisation optionnelle pour pré-remplir la ville).
2. **Mon profil** — formulaire + upload CV (glisser-déposer), ré-analyse à la demande.
3. **Tableau de bord** — résumé du profil IA, catégories détectées, mots-clés, top matchs.
4. **Recherche** — barre de recherche, filtres (lieu, télétravail, contrat, salaire, séniorité),
   carte + liste, sources interrogées en direct.
5. **Offres** — liste + détail avec **explication du match** (pourquoi ça colle).
6. **Candidatures** — kanban/suivi de statut.

### Services navigateur utilisés
- **Géolocalisation** (`navigator.geolocation`) → pré-remplissage ville + recherche locale.
- **Notifications** (`Notification`) → alerte fin de recherche / nouveaux matchs.
- **Clipboard**, **Web Share API**, **localStorage** (préférences UI & dernier profil).
- **File API / Drag & Drop** pour l'upload du CV.

---

## 3. Architecture technique

```
                 ┌───────────────────────────────────────────┐
   Navigateur →  │  Axum (API REST) + static SPA (web/)       │
                 └───────────────────┬───────────────────────┘
                                     │
        ┌────────────────────────────┼─────────────────────────────┐
        ▼                            ▼                             ▼
┌───────────────┐          ┌─────────────────┐          ┌──────────────────┐
│  Agents IA    │          │   Scraper       │          │   Persistance    │
│ (orchestrator)│          │  multi-sources  │          │   SQLite (SQLx)  │
├───────────────┤          ├─────────────────┤          ├──────────────────┤
│ CV Analyzer   │          │ Arbeitnow (API) │          │ profiles         │
│ Profile Cat.  │          │ Remotive (API)  │          │ jobs             │
│ Job Cat.      │          │ RemoteOK (API)  │          │ matches          │
│ Matcher       │          │ Generic HTML    │          │ applications     │
│ Orchestrator  │          │ + 10 autres     │          │ search_runs      │
└───────┬───────┘          └─────────────────┘          └──────────────────┘
        │
        ▼
┌───────────────────────────────────────────┐
│ LLM Client (pluggable)                    │
│  • Ollama (local, par défaut)             │
│  • OpenAI-compatible (cloud)              │
│  • Heuristiques (fallback, zéro dépendance)│
└───────────────────────────────────────────┘
```

### Choix technologiques
| Couche       | Techno                              | Pourquoi                                   |
|--------------|-------------------------------------|---------------------------------------------|
| Langage      | Rust (édition 2021)                 | Vitesse, sûreté, concurrence                |
| Web          | Axum 0.7 + Tower                    | Léger, async, très rapide                   |
| DB           | SQLite via SQLx (WAL)               | Zéro installation, portable                 |
| Scraping     | reqwest + scraper (CSS selectors)   | HTTP natif + parsing HTML robuste           |
| IA           | Client LLM pluggable + fallback     | Marche **avec ou sans** clé/LLM             |
| Auth         | Argon2id + JWT (jsonwebtoken)       | Comptes multi-utilisateurs sécurisés        |
| Front        | React + TS (Vite/Tailwind/shadcn/ui)| UX premium, build → `web/` servi par Axum   |

### Sources d'offres (13)
- **API JSON** : Arbeitnow, Remotive, RemoteOK, Jobicy, Himalayas, **Portal Job Madagascar**, Welcome to the Jungle (Algolia), Adzuna 🔑, Jooble 🔑.
- **RSS** : We Work Remotely (parseur `quick-xml`).
- **HTML (scraper)** : LinkedIn (endpoint public, rate-limité), HelloWork, Tanitjobs.
- 🔑 = clé API optionnelle (`ADZUNA_*`, `JOOBLE_API_KEY`). Chaque source est isolée : une erreur (429, 5xx, site SPA) n'empêche pas les autres et est remontée via `source_errors`.
- **Non supportés volontairement** : Indeed, Glassdoor (anti-bot Cloudflare + CGU).

### Pourquoi si rapide ?
- **Recherche parallèle** : toutes les sources sont interrogées en `join_all` (concurrence).
- **Cache & dédup** : les offres sont dédupliquées par `(source, external_id)` + hash de titre/entreprise.
- **Extraction CV en `spawn_blocking`** : le parsing PDF (CPU) ne bloque pas l'event loop.
- **SQLite WAL + index** sur `jobs(remote)`, `jobs(category)`, `matches(profile_id, score)`.
- **Le scoring tourne côté serveur et renvoie un top-N** (pagination).

---

## 4. Les agents

| Agent                     | Entrée                    | Sortie                                              |
|---------------------------|---------------------------|-----------------------------------------------------|
| `CvAnalyzerAgent`         | texte du CV               | profil structuré (skills, XP, formation, langues)   |
| `ProfileCategorizerAgent` | profil structuré          | catégorie principale, mots-clés, titres suggérés    |
| `JobCategorizerAgent`     | offres brutes (scraping)  | offres normalisées + catégorie + skills + séniorité |
| `MatcherAgent`            | profil + offres           | score 0-100 + raisons explicables                   |
| `SearchOrchestratorAgent` | profil + préférences      | pipeline complet, persiste résultats                |

Chaque agent utilise le **LLM client** s'il est disponible, sinon bascule sur des
**heuristiques** (regex + dictionnaires) — l'application reste 100 % fonctionnelle hors-ligne.

---

## 5. Modèle de données

- `users` : compte (email unique, nom, mot de passe haché Argon2id).
- `profiles` : rattaché à un `user` (`user_id`), nommé (`name`), avec un profil `is_default`.
  Identité, résumé, séniorité, années d'XP, langues/formation/expériences (JSON),
  compétences, domaines, titres cibles, préférences (télétravail, contrat, salaire).
- `jobs` : source, external_id, titre, entreprise, lieu, remote, description, URL, salaire,
  contrat, séniorité, catégorie, skills, dates, brut (JSON).
- `matches` : (profil, offre, score, raisons).
- `applications` : suivi de candidature (statut, notes).
- `search_runs` : historique des recherches (requête, lieu, sources, nb résultats).

Chaque donnée métier (profil, recherche, match, candidature) est **isolée par profil**,
et chaque profil appartient à un **utilisateur authentifié**.

---

## 6. API REST

Authentification JWT : `Authorization: Bearer <token>` (ou cookie `workeo_token`).

| Méthode | Route                              | Description                          |
|---------|------------------------------------|--------------------------------------|
| POST    | `/api/auth/register`               | Créer un compte                      |
| POST    | `/api/auth/login`                  | Connexion (renvoie un JWT)           |
| GET     | `/api/auth/me`                     | Utilisateur courant                  |
| POST    | `/api/auth/logout`                 | Effacer le cookie de session         |
| GET     | `/api/profiles`                    | Lister les profils de l'utilisateur  |
| POST    | `/api/profiles`                    | Créer un profil                      |
| GET     | `/api/profiles/:id`                | Lire un profil                       |
| PUT     | `/api/profiles/:id`                | Mettre à jour                        |
| POST    | `/api/profiles/:id/cv`             | Upload CV (multipart) + analyse IA   |
| POST    | `/api/profiles/:id/analyze`        | (Ré)analyser le profil               |
| POST    | `/api/search`                      | Lancer une recherche multi-agents    |
| GET     | `/api/profiles/:id/recommendations`| Top offres recommandées              |
| GET    | `/api/jobs/:id`                    | Détail d'une offre                   |
| POST    | `/api/applications`                | Créer une candidature                |
| GET     | `/api/profiles/:id/applications`   | Suivi                                |
| GET    | `/api/health`                      | État + config IA détectée            |

---

## 7. Configuration (.env)

```
PORT=8080
DATABASE_URL=sqlite://data/workeo.db
LLM_PROVIDER=ollama            # ollama | openai | none
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama3.1
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_API_KEY=
OPENAI_MODEL=gpt-4o-mini
JWT_SECRET=change-me-in-production
```

`LLM_PROVIDER=none` (ou LLM injoignable) → bascule automatique en mode heuristique.

---

## 8. Démarrage

```bash
cp .env.example .env
cargo run --release
# → http://localhost:8080
```

L'interface compilée est fournie dans `web/`. Pour la reconstruire :

```bash
cd frontend && pnpm install && pnpm build   # génère ../web
```

Aucun service externe requis. Optionnel : Ollama pour l'IA locale.
