//! Système multi-agents de Workeo.
//!
//! | Agent                     | Rôle                                                    |
//! |---------------------------|---------------------------------------------------------|
//! | [`cv_analyzer`]           | Extrait les infos structurées d'un CV                    |
//! | [`profile_categorizer`]   | Catégorise le profil et génère mots-clés / titres cibles |
//! | [`job_categorizer`]       | Nettoie et catégorise les offres issues du scraping      |
//! | [`matcher`]               | Score la compatibilité profil ↔ offres                   |
//! | [`orchestrator`]          | Coordonne le pipeline complet                            |
//! | [`llm`]                   | Client LLM unifié (Ollama / OpenAI / heuristiques)       |
//! | [`heuristics`]            | Fallback local sans LLM                                  |

pub mod cv_analyzer;
pub mod heuristics;
pub mod job_categorizer;
pub mod llm;
pub mod matcher;
pub mod orchestrator;
pub mod profile_categorizer;
