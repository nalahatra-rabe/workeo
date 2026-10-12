use futures::future::join_all;
use reqwest::Client;

use super::{source_by_name, JobSource};
use crate::models::RawJob;

/// Résultat agrégé d'une interrogation multi-sources.
pub struct FetchOutcome {
    pub jobs: Vec<RawJob>,
    pub errors: Vec<String>,
}

/// Interroge toutes les sources demandées **en parallèle** et agrège les offres.
///
/// Les erreurs d'une source n'interrompent pas les autres : elles sont
/// collectées dans `errors` pour être remontées à l'utilisateur.
pub async fn fetch_all(
    http: &Client,
    source_names: &[String],
    query: &str,
    location: Option<&str>,
    limit_per_source: usize,
) -> FetchOutcome {
    let sources: Vec<Box<dyn JobSource>> = source_names
        .iter()
        .filter_map(|n| source_by_name(n))
        .collect();

    let futures = sources.iter().map(|src| {
        let name = src.name();
        let label = src.label();
        let fut = src.fetch(http, query, location, limit_per_source);
        async move {
            match fut.await {
                Ok(jobs) => {
                    tracing::info!(source = label, count = jobs.len(), "source OK");
                    (jobs, None)
                }
                Err(e) => {
                    let msg = format!("{name}: {e}");
                    tracing::warn!("{msg}");
                    (Vec::new(), Some(msg))
                }
            }
        }
    });

    let results = join_all(futures).await;
    let mut jobs = Vec::new();
    let mut errors = Vec::new();
    for (mut js, err) in results {
        jobs.append(&mut js);
        if let Some(e) = err {
            errors.push(e);
        }
    }

    FetchOutcome { jobs, errors }
}
