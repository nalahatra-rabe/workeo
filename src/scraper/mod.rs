//! Module de scraping web multi-sources.
//!
//! Chaque source implémente le trait [`JobSource`]. Le moteur interroge toutes
//! les sources **en parallèle** (`join_all`) et agrège les offres brutes.

pub mod engine;
pub mod sources;

use async_trait::async_trait;
use reqwest::Client;

use crate::models::RawJob;

pub use engine::fetch_all;

/// Une source d'offres d'emploi interrogeable.
#[async_trait]
pub trait JobSource: Send + Sync {
    /// Nom technique de la source (clé de configuration).
    fn name(&self) -> &'static str;

    /// Libellé lisible pour l'UI.
    fn label(&self) -> &'static str;

    /// Récupère un lot d'offres brutes.
    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>>;
}

/// Construit une source à partir de son nom.
pub fn source_by_name(name: &str) -> Option<Box<dyn JobSource>> {
    match name.trim().to_lowercase().as_str() {
        "arbeitnow" => Some(Box::new(sources::ArbeitnowSource)),
        "remotive" => Some(Box::new(sources::RemotiveSource)),
        "remoteok" => Some(Box::new(sources::RemoteOkSource)),
        "demo" => Some(Box::new(sources::DemoSource)),
        _ => None,
    }
}

/// Liste des sources connues (nom, libellé).
pub fn known_sources() -> Vec<(&'static str, &'static str)> {
    vec![
        ("arbeitnow", "Arbeitnow"),
        ("remotive", "Remotive"),
        ("remoteok", "RemoteOK"),
        ("demo", "Démo (hors-ligne)"),
    ]
}
