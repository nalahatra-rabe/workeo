//! Sources d'offres concrètes.
//!
//! - **Arbeitnow**, **Remotive**, **Jobicy**, **Himalayas** : API JSON publiques.
//! - **RemoteOK** : API JSON publique (offres full-remote).
//! - **We Work Remotely** : flux RSS public.
//! - **Portal Job Madagascar**, **Tanitjobs** : offres Madagascar (scraping).
//!
//! Toutes renvoient des [`RawJob`] ; la normalisation est faite ensuite par le
//! `JobCategorizerAgent`.

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::header::HeaderMap;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;
use std::env;

use super::JobSource;
use crate::models::RawJob;

// ─────────────────────────────────────────────────────────────
// Arbeitnow — https://www.arbeitnow.com/api/job-board-api
// ─────────────────────────────────────────────────────────────

pub struct ArbeitnowSource;

#[async_trait]
impl JobSource for ArbeitnowSource {
    fn name(&self) -> &'static str {
        "arbeitnow"
    }
    fn label(&self) -> &'static str {
        "Arbeitnow"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let url = "https://www.arbeitnow.com/api/job-board-api";
        let resp = http.get(url).send().await?.error_for_status()?;
        let json: Value = resp.json().await?;
        let data = json.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();

        let q = query.to_lowercase();
        let loc = location.map(|l| l.to_lowercase());
        let mut out = Vec::new();

        for item in data {
            let title = s(&item, "title");
            let company = s(&item, "company_name");
            if title.is_empty() {
                continue;
            }
            let haystack = format!("{title} {company} {}", s(&item, "description")).to_lowercase();
            if !q.is_empty() && !q.split_whitespace().any(|w| haystack.contains(w)) {
                continue;
            }
            let job_loc = s(&item, "location");
            if let Some(l) = &loc {
                if !l.is_empty() && !job_loc.to_lowercase().contains(l) {
                    continue;
                }
            }
            let tags = item
                .get("tags")
                .and_then(|t| t.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();

            out.push(RawJob {
                source: "arbeitnow".into(),
                external_id: s(&item, "slug"),
                title,
                company,
                location: opt(job_loc),
                description: opt(s(&item, "description")),
                url: opt(s(&item, "url")),
                remote: item.get("remote").and_then(|r| r.as_bool()),
                salary: None,
                contract_type: item
                    .get("job_types")
                    .and_then(|t| t.as_array())
                    .and_then(|a| a.first())
                    .and_then(|x| x.as_str())
                    .map(String::from),
                tags,
                posted_at: item
                    .get("created_at")
                    .and_then(|c| c.as_i64())
                    .and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            });

            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Remotive — https://remotive.com/api/remote-jobs
// ─────────────────────────────────────────────────────────────

pub struct RemotiveSource;

#[async_trait]
impl JobSource for RemotiveSource {
    fn name(&self) -> &'static str {
        "remotive"
    }
    fn label(&self) -> &'static str {
        "Remotive"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let mut url = format!("https://remotive.com/api/remote-jobs?limit={limit}");
        if !query.is_empty() {
            url.push_str(&format!("&search={}", urlencode(query)));
        }
        let resp = http.get(&url).send().await?.error_for_status()?;
        let json: Value = resp.json().await?;
        let jobs = json.get("jobs").and_then(|d| d.as_array()).cloned().unwrap_or_default();

        let loc = location.map(|l| l.to_lowercase());
        let mut out = Vec::new();
        for item in jobs {
            let title = s(&item, "title");
            if title.is_empty() {
                continue;
            }
            let job_loc = s(&item, "candidate_required_location");
            if let Some(l) = &loc {
                if !l.is_empty() && !job_loc.to_lowercase().contains(l) {
                    continue;
                }
            }
            out.push(RawJob {
                source: "remotive".into(),
                external_id: item.get("id").map(|v| v.to_string()).unwrap_or_else(|| title.clone()),
                title,
                company: s(&item, "company_name"),
                location: opt(job_loc),
                description: opt(strip_html(&s(&item, "description"))),
                url: opt(s(&item, "url")),
                remote: Some(true),
                salary: opt(s(&item, "salary")),
                contract_type: opt(s(&item, "job_type")),
                tags: item
                    .get("tags")
                    .and_then(|t| t.as_array())
                    .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                    .unwrap_or_default(),
                posted_at: s(&item, "publication_date")
                    .parse::<DateTime<Utc>>()
                    .ok(),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// RemoteOK — https://remoteok.com/api
// ─────────────────────────────────────────────────────────────

pub struct RemoteOkSource;

#[async_trait]
impl JobSource for RemoteOkSource {
    fn name(&self) -> &'static str {
        "remoteok"
    }
    fn label(&self) -> &'static str {
        "RemoteOK"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        _location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let resp = http
            .get("https://remoteok.com/api")
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?;
        let json: Value = resp.json().await?;
        let arr = json.as_array().cloned().unwrap_or_default();

        let q = query.to_lowercase();
        let mut out = Vec::new();
        for item in arr {
            // Le premier élément est une notice légale sans "position".
            let title = s(&item, "position");
            if title.is_empty() {
                continue;
            }
            let tags: Vec<String> = item
                .get("tags")
                .and_then(|t| t.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let haystack = format!("{title} {} {}", s(&item, "company"), tags.join(" ")).to_lowercase();
            if !q.is_empty() && !q.split_whitespace().any(|w| haystack.contains(w)) {
                continue;
            }
            let salary = match (
                item.get("salary_min").and_then(|v| v.as_i64()),
                item.get("salary_max").and_then(|v| v.as_i64()),
            ) {
                (Some(mn), Some(mx)) if mn > 0 => Some(format!("{mn}-{mx} USD")),
                _ => None,
            };
            out.push(RawJob {
                source: "remoteok".into(),
                external_id: item
                    .get("id")
                    .map(|v| v.to_string())
                    .or_else(|| opt(s(&item, "slug")))
                    .unwrap_or_else(|| title.clone()),
                title,
                company: s(&item, "company"),
                location: opt(s(&item, "location")).or(Some("Remote".into())),
                description: opt(strip_html(&s(&item, "description"))),
                url: opt(s(&item, "url")),
                remote: Some(true),
                salary,
                contract_type: None,
                tags,
                posted_at: s(&item, "date").parse::<DateTime<Utc>>().ok(),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Jobicy — https://jobicy.com/api/v2/remote-jobs  (API JSON)
// ─────────────────────────────────────────────────────────────

pub struct JobicySource;

#[async_trait]
impl JobSource for JobicySource {
    fn name(&self) -> &'static str {
        "jobicy"
    }
    fn label(&self) -> &'static str {
        "Jobicy"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let count = limit.clamp(1, 50);
        let url = format!("https://jobicy.com/api/v2/remote-jobs?count={count}");
        let json: Value = http.get(&url).send().await?.error_for_status()?.json().await?;
        let jobs = json.get("jobs").and_then(|j| j.as_array()).cloned().unwrap_or_default();

        let mut out = Vec::new();
        for item in jobs {
            let title = s(&item, "jobTitle");
            if title.is_empty() {
                continue;
            }
            let company = s(&item, "companyName");
            let geo = s(&item, "jobGeo");
            let description = strip_html(&s(&item, "jobDescription"));
            if !query_matches(query, &format!("{title} {company} {description}")) {
                continue;
            }
            if !location_matches(location, &geo, true) {
                continue;
            }
            out.push(RawJob {
                source: "jobicy".into(),
                external_id: item
                    .get("id")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| title.clone()),
                title,
                company,
                location: opt(geo),
                description: opt(description),
                url: opt(s(&item, "url")),
                remote: Some(true),
                salary: salary_range(
                    item.get("salaryMin"),
                    item.get("salaryMax"),
                    &s(&item, "salaryCurrency"),
                ),
                contract_type: first_str(&item, "jobType"),
                tags: str_array(&item, "jobIndustry"),
                posted_at: s(&item, "pubDate").parse::<DateTime<Utc>>().ok(),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Himalayas — https://himalayas.app/jobs/api  (API JSON)
// ─────────────────────────────────────────────────────────────

pub struct HimalayasSource;

#[async_trait]
impl JobSource for HimalayasSource {
    fn name(&self) -> &'static str {
        "himalayas"
    }
    fn label(&self) -> &'static str {
        "Himalayas"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let lim = limit.clamp(1, 100);
        let url = format!("https://himalayas.app/jobs/api?limit={lim}&offset=0");
        let json: Value = http.get(&url).send().await?.error_for_status()?.json().await?;
        let jobs = json.get("jobs").and_then(|j| j.as_array()).cloned().unwrap_or_default();

        let mut out = Vec::new();
        for item in jobs {
            let title = s(&item, "title");
            if title.is_empty() {
                continue;
            }
            let company = s(&item, "companyName");
            let description = strip_html(&s(&item, "description"));
            if !query_matches(query, &format!("{title} {company} {description}")) {
                continue;
            }
            let locations = str_array(&item, "locationRestrictions");
            let job_loc = if locations.is_empty() {
                "Remote".to_string()
            } else {
                locations.join(", ")
            };
            if !location_matches(location, &job_loc, true) {
                continue;
            }
            let url_field = {
                let app = s(&item, "applicationLink");
                if app.is_empty() { s(&item, "guid") } else { app }
            };
            out.push(RawJob {
                source: "himalayas".into(),
                external_id: {
                    let g = s(&item, "guid");
                    if g.is_empty() { title.clone() } else { g }
                },
                title,
                company,
                location: opt(job_loc),
                description: opt(description),
                url: opt(url_field),
                remote: Some(true),
                salary: salary_range(
                    item.get("minSalary"),
                    item.get("maxSalary"),
                    &s(&item, "currency"),
                ),
                contract_type: opt(s(&item, "employmentType")),
                tags: str_array(&item, "categories"),
                posted_at: item
                    .get("pubDate")
                    .and_then(|v| v.as_i64())
                    .and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// We Work Remotely — https://weworkremotely.com/remote-jobs.rss
// ─────────────────────────────────────────────────────────────

pub struct WeWorkRemotelySource;

#[async_trait]
impl JobSource for WeWorkRemotelySource {
    fn name(&self) -> &'static str {
        "weworkremotely"
    }
    fn label(&self) -> &'static str {
        "We Work Remotely"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let body = http
            .get("https://weworkremotely.com/remote-jobs.rss")
            .header("Accept", "application/rss+xml, application/xml;q=0.9, */*;q=0.8")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let mut out = Vec::new();
        for item in parse_rss(&body) {
            let raw_title = item.title.trim().to_string();
            if raw_title.is_empty() {
                continue;
            }
            // Format WWR : « Entreprise: Intitulé du poste ».
            let (company, title) = match raw_title.split_once(": ") {
                Some((c, r)) => (c.trim().to_string(), r.trim().to_string()),
                None => (String::new(), raw_title.clone()),
            };
            let description = strip_html(&item.description);
            if !query_matches(query, &format!("{title} {company} {description}")) {
                continue;
            }
            if !location_matches(location, &item.region, true) {
                continue;
            }
            out.push(RawJob {
                source: "weworkremotely".into(),
                external_id: if item.guid.trim().is_empty() {
                    item.link.clone()
                } else {
                    item.guid.clone()
                },
                title,
                company,
                location: opt(if item.region.trim().is_empty() {
                    "Remote".to_string()
                } else {
                    item.region.clone()
                }),
                description: opt(strip_html(&item.description)),
                url: opt(item.link.clone()),
                remote: Some(true),
                salary: None,
                contract_type: opt(item.category),
                tags: Vec::new(),
                posted_at: DateTime::parse_from_rfc2822(item.pub_date.trim())
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
                    .or_else(|| {
                        item.pub_date
                            .trim()
                            .parse::<DateTime<Utc>>()
                            .ok()
                    }),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// We Work Remotely — parseur RSS léger (quick-xml)
// ─────────────────────────────────────────────────────────────

#[derive(Default)]
struct RssItem {
    title: String,
    link: String,
    region: String,
    category: String,
    description: String,
    pub_date: String,
    guid: String,
}

fn parse_rss(xml: &str) -> Vec<RssItem> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut cur: Option<RssItem> = None;
    let mut field: Option<&'static str> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"item" => cur = Some(RssItem::default()),
                b"title" => field = Some("title"),
                b"link" => field = Some("link"),
                b"region" => field = Some("region"),
                b"category" => field = Some("category"),
                b"description" => field = Some("description"),
                b"pubDate" => field = Some("pubDate"),
                b"guid" => field = Some("guid"),
                _ => field = None,
            },
            Ok(Event::Text(e)) => {
                if let (Some(item), Some(f)) = (cur.as_mut(), field) {
                    if let Ok(txt) = e.unescape() {
                        push_field(item, f, &txt);
                    }
                }
            }
            Ok(Event::CData(e)) => {
                if let (Some(item), Some(f)) = (cur.as_mut(), field) {
                    push_field(item, f, &String::from_utf8_lossy(&e));
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"item" {
                    if let Some(it) = cur.take() {
                        items.push(it);
                    }
                }
                field = None;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    items
}

fn push_field(item: &mut RssItem, field: &str, txt: &str) {
    let target = match field {
        "title" => &mut item.title,
        "link" => &mut item.link,
        "region" => &mut item.region,
        "category" => &mut item.category,
        "description" => &mut item.description,
        "pubDate" => &mut item.pub_date,
        "guid" => &mut item.guid,
        _ => return,
    };
    if field == "category" && !target.is_empty() {
        target.push_str(", ");
    }
    target.push_str(txt);
}

// ─────────────────────────────────────────────────────────────
// Portal Job Madagascar — API JSON Laravel (POST + session XSRF)
// GET /emploi/liste (cookies) puis POST /api/emploi/annonces
// ─────────────────────────────────────────────────────────────

const PJM_BASE: &str = "https://www.portaljob-madagascar.com";

pub struct PortalJobMgSource;

#[async_trait]
impl JobSource for PortalJobMgSource {
    fn name(&self) -> &'static str {
        "portaljob_mg"
    }
    fn label(&self) -> &'static str {
        "Portal Job Madagascar"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let listing = format!("{PJM_BASE}/emploi/liste");

        // 1) Amorcer la session pour récupérer les cookies (dont XSRF-TOKEN).
        let mut jar = CookieJar::default();
        let session = http
            .get(&listing)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .send()
            .await?
            .error_for_status()?;
        jar.update_from(session.headers());

        let city = location.map(city_of).unwrap_or_default();
        let per_page = 12usize; // fixé côté serveur
        let pages = limit.div_ceil(per_page).max(1);
        let mut out = Vec::new();

        for page in 1..=pages {
            let xsrf = jar.get("XSRF-TOKEN").map(percent_decode).unwrap_or_default();
            let body = serde_json::json!({
                "query": query,
                "ville": city,
                "contrats": [],
                "secteurs": [],
                "niveaux": [],
                "experiences": [],
                "international": 0,
                "page": page,
            });

            let mut rb = http
                .post(format!("{PJM_BASE}/api/emploi/annonces"))
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Origin", PJM_BASE)
                .header("Referer", &listing)
                .json(&body);
            if !xsrf.is_empty() {
                rb = rb.header("X-XSRF-TOKEN", xsrf);
            }
            let cookies = jar.header();
            if !cookies.is_empty() {
                rb = rb.header("Cookie", cookies);
            }

            let resp = rb.send().await?;
            jar.update_from(resp.headers());
            let status = resp.status();
            if !status.is_success() {
                if page == 1 {
                    anyhow::bail!("HTTP {status}");
                }
                break;
            }
            let json: Value = resp.json().await?;

            // Les offres « prime » / « partenaires » n'apparaissent qu'en page 1.
            let mut lists: Vec<Vec<Value>> = Vec::new();
            if page == 1 {
                for key in ["annonces_prime", "annonces_partenaires"] {
                    if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
                        lists.push(arr.clone());
                    }
                }
            }
            if let Some(arr) = json.pointer("/annonces/data").and_then(|v| v.as_array()) {
                lists.push(arr.clone());
            }

            let mut page_items = 0usize;
            for list in lists {
                for item in list {
                    let title = s(&item, "annonce_poste");
                    if title.is_empty() {
                        continue;
                    }
                    let slug = s(&item, "annonce_url");
                    let contract = item
                        .pointer("/annonce_contrat/type_contrat")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                        .or_else(|| opt(s(&item, "annonce_contrat_txt")));
                    let sector = item
                        .pointer("/secteur_emploi/secteur")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    out.push(RawJob {
                        source: "portaljob_mg".into(),
                        external_id: slug.clone(),
                        title,
                        company: s(&item, "annonce_entreprise"),
                        location: opt(s(&item, "annonce_ville")),
                        description: opt(s(&item, "annonce_desc_poste")),
                        url: if slug.is_empty() {
                            None
                        } else {
                            Some(format!("{PJM_BASE}/emploi/view/{slug}"))
                        },
                        remote: None,
                        salary: None,
                        contract_type: contract,
                        tags: sector.into_iter().collect(),
                        posted_at: s(&item, "annonce_date").parse::<DateTime<Utc>>().ok(),
                    });
                    page_items += 1;
                    if out.len() >= limit {
                        return Ok(out);
                    }
                }
            }

            let last_page = json
                .pointer("/annonces/last_page")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as usize;
            if page_items == 0 || page >= last_page {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Welcome to the Jungle — index Algolia public (JSON)
// POST /1/indexes/wk_cms_jobs_production/query (Referer requis)
// ─────────────────────────────────────────────────────────────

const WTTJ_ALGOLIA_APP: &str = "CSEKHVMS53";
const WTTJ_ALGOLIA_KEY: &str = "4bd8f6215d0cc52b26430765769e65a0";
const WTTJ_INDEX: &str = "wk_cms_jobs_production";

pub struct WttjSource;

#[async_trait]
impl JobSource for WttjSource {
    fn name(&self) -> &'static str {
        "wttj"
    }
    fn label(&self) -> &'static str {
        "Welcome to the Jungle"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let url = format!(
            "https://{WTTJ_ALGOLIA_APP}-dsn.algolia.net/1/indexes/{WTTJ_INDEX}/query"
        );
        let body = serde_json::json!({
            "query": query,
            "hitsPerPage": limit.clamp(1, 100),
            "page": 0,
        });
        let json: Value = http
            .post(&url)
            .header("X-Algolia-Application-Id", WTTJ_ALGOLIA_APP)
            .header("X-Algolia-API-Key", WTTJ_ALGOLIA_KEY)
            .header("Content-Type", "application/json")
            .header("Referer", "https://www.welcometothejungle.com/")
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let hits = json.get("hits").and_then(|h| h.as_array()).cloned().unwrap_or_default();
        let mut out = Vec::new();
        for hit in hits {
            let title = s(&hit, "name");
            if title.is_empty() {
                continue;
            }
            let company = hit
                .pointer("/organization/name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let org_slug = hit
                .pointer("/organization/slug")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let slug = s(&hit, "slug");

            let city = hit.pointer("/office/city").and_then(|v| v.as_str()).unwrap_or("");
            let country = hit.pointer("/office/country").and_then(|v| v.as_str()).unwrap_or("");
            let job_loc = [city, country]
                .iter()
                .filter(|p| !p.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");

            let remote = match s(&hit, "remote").as_str() {
                "fulltime" => Some(true),
                "no" => Some(false),
                _ => None,
            };
            if !location_matches(location, &job_loc, remote.unwrap_or(false)) {
                continue;
            }
            out.push(RawJob {
                source: "wttj".into(),
                external_id: slug.clone(),
                title,
                company,
                location: opt(job_loc),
                description: None,
                url: if slug.is_empty() || org_slug.is_empty() {
                    None
                } else {
                    Some(format!(
                        "https://www.welcometothejungle.com/fr/companies/{org_slug}/jobs/{slug}"
                    ))
                },
                remote,
                salary: salary_range(
                    hit.get("salary_minimum"),
                    hit.get("salary_maximum"),
                    &s(&hit, "salary_currency"),
                ),
                contract_type: hit
                    .pointer("/contract_type_names/fr")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                tags: str_array(&hit, "sectors"),
                posted_at: s(&hit, "published_at").parse::<DateTime<Utc>>().ok(),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// HelloWork (France) — scraping HTML, sélecteurs `data-cy` stables
// ─────────────────────────────────────────────────────────────

pub struct HelloWorkSource;

#[async_trait]
impl JobSource for HelloWorkSource {
    fn name(&self) -> &'static str {
        "hellowork"
    }
    fn label(&self) -> &'static str {
        "HelloWork"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let per_page = 30usize;
        let pages = limit.div_ceil(per_page).max(1);
        let mut out = Vec::new();

        for page in 1..=pages {
            let mut url = format!(
                "https://www.hellowork.com/fr-fr/emploi/recherche.html?k={}&p={page}",
                urlencode(query)
            );
            if let Some(l) = location.map(str::trim).filter(|l| !l.is_empty()) {
                url.push_str(&format!("&l={}", urlencode(l)));
            }
            let html = http
                .get(&url)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                .header("Accept-Language", "fr-FR,fr;q=0.9")
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?;
            let doc = Html::parse_document(&html);
            let Some(card_sel) = sel_opt("[data-cy=\"serpCard\"]") else {
                break;
            };

            let mut found = 0usize;
            for card in doc.select(&card_sel) {
                let link_sel = "a[data-cy=\"offerTitle\"]";
                let title = {
                    let t = first_text(&card, link_sel);
                    if t.is_empty() { first_text(&card, "a[data-cy=\"offerTitle\"] h3") } else { t }
                };
                if title.is_empty() {
                    continue;
                }
                let href = first_attr(&card, link_sel, "href");
                let job_url = absolute("https://www.hellowork.com", &href);
                let mut company = first_attr(&card, "input[name=\"company\"]", "value");
                if company.is_empty() {
                    let t = first_attr(&card, link_sel, "title");
                    company = t.rsplit_once(" - ").map(|(_, c)| c.to_string()).unwrap_or_default();
                }
                let job_loc = first_text(&card, "[data-cy=\"localisationCard\"]");
                let contract = all_text(&card, "[data-cy=\"contractTag\"]").join(", ");
                out.push(RawJob {
                    source: "hellowork".into(),
                    external_id: href.clone(),
                    title,
                    company,
                    location: opt(job_loc),
                    description: None,
                    url: job_url,
                    remote: None,
                    salary: None,
                    contract_type: opt(contract),
                    tags: Vec::new(),
                    posted_at: None,
                });
                found += 1;
                if out.len() >= limit {
                    break;
                }
            }
            if found == 0 || out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Tanitjobs — scraping HTML (Tunisie / francophonie)
// ─────────────────────────────────────────────────────────────

pub struct TanitjobsSource;

#[async_trait]
impl JobSource for TanitjobsSource {
    fn name(&self) -> &'static str {
        "tanitjobs"
    }
    fn label(&self) -> &'static str {
        "Tanitjobs"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let mut out = Vec::new();
        let pages = limit.div_ceil(20).max(1);

        for page in 1..=pages {
            let mut url = format!(
                "https://www.tanitjobs.com/jobs/?keywords[all_words]={}&page={page}&action=search&listing_type[equal]=Job",
                urlencode(query)
            );
            if let Some(l) = location.map(str::trim).filter(|l| !l.is_empty()) {
                url.push_str(&format!("&GooglePlace[location][value]={}", urlencode(l)));
            }
            let html = http
                .get(&url)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?;
            let doc = Html::parse_document(&html);
            let Some(card_sel) = sel_opt("div.sj-job-card") else {
                break;
            };

            let mut found = 0usize;
            for card in doc.select(&card_sel) {
                let title = first_text(&card, ".sj-card-title a");
                if title.is_empty() {
                    continue;
                }
                let href = first_attr(&card, ".sj-card-title a", "href");
                let job_loc = first_text(&card, ".sj-card-tag.sj-loc");
                let contract = first_text(&card, ".sj-card-tag.sj-type");
                let worktype = first_text(&card, ".sj-card-tag.sj-worktype");
                let wt = worktype.to_lowercase();
                let remote = if wt.contains("télétravail")
                    || wt.contains("teletravail")
                    || wt.contains("remote")
                {
                    Some(true)
                } else {
                    None
                };
                out.push(RawJob {
                    source: "tanitjobs".into(),
                    external_id: href.clone(),
                    title,
                    company: first_text(&card, ".sj-card-company"),
                    location: opt(job_loc),
                    description: opt(first_text(&card, ".sj-card-desc")),
                    url: absolute("https://www.tanitjobs.com", &href),
                    remote,
                    salary: None,
                    contract_type: opt(contract),
                    tags: if worktype.is_empty() { Vec::new() } else { vec![worktype] },
                    posted_at: None,
                });
                found += 1;
                if out.len() >= limit {
                    break;
                }
            }
            if found == 0 || out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// LinkedIn Jobs — endpoint « guest » public (best effort, rate-limité)
// ─────────────────────────────────────────────────────────────

pub struct LinkedInSource;

#[async_trait]
impl JobSource for LinkedInSource {
    fn name(&self) -> &'static str {
        "linkedin"
    }
    fn label(&self) -> &'static str {
        "LinkedIn"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let loc = location.map(str::trim).filter(|l| !l.is_empty()).unwrap_or("");
        let mut out = Vec::new();
        let max_requests = 3usize;

        for i in 0..max_requests {
            let start = i * 10;
            let url = format!(
                "https://www.linkedin.com/jobs-guest/jobs/api/seeMoreJobPostings/search?keywords={}&location={}&start={start}",
                urlencode(query),
                urlencode(loc)
            );
            let resp = http
                .get(&url)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                .header("Accept-Language", "en-US,en;q=0.9")
                .send()
                .await?;
            if !resp.status().is_success() {
                // 429 : rate limit. On renvoie ce qu'on a déjà (best effort).
                if out.is_empty() {
                    anyhow::bail!("limite de débit atteinte (HTTP {})", resp.status());
                }
                break;
            }
            let html = resp.text().await?;
            let doc = Html::parse_document(&html);
            let Some(card_sel) = sel_opt("div.base-search-card.job-search-card") else {
                break;
            };

            let mut found = 0usize;
            for card in doc.select(&card_sel) {
                let title = first_text(&card, ".base-search-card__title");
                if title.is_empty() {
                    continue;
                }
                let urn = first_attr(&card, "[data-entity-urn]", "data-entity-urn");
                let external_id = urn.rsplit(':').next().unwrap_or("").to_string();
                let href = first_attr(&card, "a.base-card__full-link", "href");
                let href = href.split('?').next().unwrap_or("").to_string();
                out.push(RawJob {
                    source: "linkedin".into(),
                    external_id: if external_id.is_empty() {
                        href.clone()
                    } else {
                        external_id
                    },
                    title,
                    company: first_text(&card, ".base-search-card__subtitle"),
                    location: opt(first_text(&card, ".job-search-card__location")),
                    description: None,
                    url: opt(href),
                    remote: None,
                    salary: None,
                    contract_type: None,
                    tags: Vec::new(),
                    posted_at: first_attr(&card, "time.job-search-card__listdate", "datetime")
                        .split('T')
                        .next()
                        .and_then(|d| format!("{d}T00:00:00Z").parse::<DateTime<Utc>>().ok()),
                });
                found += 1;
                if out.len() >= limit {
                    return Ok(out);
                }
            }
            if found == 0 {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Adzuna — API officielle (clé: ADZUNA_APP_ID / ADZUNA_APP_KEY)
// ─────────────────────────────────────────────────────────────

pub struct AdzunaSource;

#[async_trait]
impl JobSource for AdzunaSource {
    fn name(&self) -> &'static str {
        "adzuna"
    }
    fn label(&self) -> &'static str {
        "Adzuna"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let app_id = env::var("ADZUNA_APP_ID").ok().filter(|v| !v.is_empty());
        let app_key = env::var("ADZUNA_APP_KEY").ok().filter(|v| !v.is_empty());
        let (Some(app_id), Some(app_key)) = (app_id, app_key) else {
            anyhow::bail!("ADZUNA_APP_ID / ADZUNA_APP_KEY non configurés");
        };
        let country = env::var("ADZUNA_COUNTRY").unwrap_or_else(|_| "fr".into());
        let mut url = format!(
            "https://api.adzuna.com/v1/api/jobs/{country}/search/1?app_id={app_id}&app_key={app_key}&results_per_page={}&what={}&content-type=application/json",
            limit.clamp(1, 50),
            urlencode(query)
        );
        if let Some(l) = location.map(str::trim).filter(|l| !l.is_empty()) {
            url.push_str(&format!("&where={}", urlencode(l)));
        }
        let json: Value = http.get(&url).send().await?.error_for_status()?.json().await?;
        let results = json.get("results").and_then(|r| r.as_array()).cloned().unwrap_or_default();
        let mut out = Vec::new();
        for r in results {
            let title = s(&r, "title");
            if title.is_empty() {
                continue;
            }
            let company = r.pointer("/company/display_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let job_loc = r.pointer("/location/display_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let contract = opt(s(&r, "contract_time")).or_else(|| opt(s(&r, "contract_type")));
            let category = r.pointer("/category/label").and_then(|v| v.as_str()).map(|c| vec![c.to_string()]).unwrap_or_default();
            out.push(RawJob {
                source: "adzuna".into(),
                external_id: s(&r, "id"),
                title,
                company,
                location: opt(job_loc),
                description: opt(s(&r, "description")),
                url: opt(s(&r, "redirect_url")),
                remote: None,
                salary: salary_range(r.get("salary_min"), r.get("salary_max"), ""),
                contract_type: contract,
                tags: category,
                posted_at: s(&r, "created").parse::<DateTime<Utc>>().ok(),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────
// Jooble — API officielle (clé: JOOBLE_API_KEY), POST JSON
// ─────────────────────────────────────────────────────────────

pub struct JoobleSource;

#[async_trait]
impl JobSource for JoobleSource {
    fn name(&self) -> &'static str {
        "jooble"
    }
    fn label(&self) -> &'static str {
        "Jooble"
    }

    async fn fetch(
        &self,
        http: &Client,
        query: &str,
        location: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<RawJob>> {
        let key = env::var("JOOBLE_API_KEY").ok().filter(|v| !v.is_empty());
        let Some(key) = key else {
            anyhow::bail!("JOOBLE_API_KEY non configuré");
        };
        let body = serde_json::json!({
            "keywords": query,
            "location": location.unwrap_or(""),
            "page": "1",
        });
        let resp = http
            .post(format!("https://jooble.org/api/{key}"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            anyhow::bail!("HTTP {}", resp.status());
        }
        let json: Value = resp.json().await?;
        let jobs = json.get("jobs").and_then(|j| j.as_array()).cloned().unwrap_or_default();
        let mut out = Vec::new();
        for j in jobs {
            let title = s(&j, "title");
            if title.is_empty() {
                continue;
            }
            out.push(RawJob {
                source: "jooble".into(),
                external_id: s(&j, "id"),
                title,
                company: s(&j, "company"),
                location: opt(s(&j, "location")),
                description: opt(strip_html(&s(&j, "snippet"))),
                url: opt(s(&j, "link")),
                remote: None,
                salary: opt(s(&j, "salary")),
                contract_type: opt(s(&j, "type")),
                tags: Vec::new(),
                posted_at: s(&j, "updated").parse::<DateTime<Utc>>().ok(),
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

// ─── Helpers ────────────────────────────────────────────────

/// Compile un sélecteur CSS sans jamais paniquer.
fn sel_opt(sel: &str) -> Option<Selector> {
    Selector::parse(sel).ok()
}

/// Normalise les espaces d'un texte.
fn squash(text: String) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Texte du premier élément correspondant au sélecteur.
fn first_text(el: &ElementRef, sel: &str) -> String {
    sel_opt(sel)
        .and_then(|s| el.select(&s).next())
        .map(|e| squash(e.text().collect::<String>()))
        .unwrap_or_default()
}

/// Attribut du premier élément correspondant au sélecteur.
fn first_attr(el: &ElementRef, sel: &str, attr: &str) -> String {
    sel_opt(sel)
        .and_then(|s| el.select(&s).next())
        .and_then(|e| e.value().attr(attr).map(str::to_string))
        .unwrap_or_default()
}

/// Textes de tous les éléments correspondant au sélecteur.
fn all_text(el: &ElementRef, sel: &str) -> Vec<String> {
    let Some(s) = sel_opt(sel) else {
        return Vec::new();
    };
    el.select(&s)
        .map(|e| squash(e.text().collect::<String>()))
        .filter(|t| !t.is_empty())
        .collect()
}

/// Rend une URL absolue à partir d'une base et d'un href éventuellement relatif.
fn absolute(base: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() {
        None
    } else if href.starts_with("http://") || href.starts_with("https://") {
        Some(href.to_string())
    } else if let Some(rest) = href.strip_prefix("//") {
        Some(format!("https://{rest}"))
    } else if href.starts_with('/') {
        Some(format!("{base}{href}"))
    } else {
        Some(format!("{base}/{href}"))
    }
}

/// Petit pot à cookies (sessions Laravel / jeton XSRF).
#[derive(Default)]
struct CookieJar {
    jar: std::collections::HashMap<String, String>,
}

impl CookieJar {
    fn update_from(&mut self, headers: &HeaderMap) {
        for value in headers.get_all(reqwest::header::SET_COOKIE) {
            if let Ok(raw) = value.to_str() {
                if let Some((k, v)) = raw.split(';').next().and_then(|kv| kv.split_once('=')) {
                    self.jar.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
    }
    fn get(&self, name: &str) -> Option<&str> {
        self.jar.get(name).map(String::as_str)
    }
    fn header(&self) -> String {
        self.jar
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// Décode une chaîne percent-encodée (ex. cookie XSRF-TOKEN).
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Extrait la ville d'une localisation libre (« Tananarive, Madagascar » → « Tananarive »).
fn city_of(location: &str) -> String {
    location.split(',').next().unwrap_or(location).trim().to_string()
}

/// Vrai si la requête est vide ou si au moins un de ses mots apparaît.
fn query_matches(query: &str, haystack: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    let hay = haystack.to_lowercase();
    q.split_whitespace().any(|w| hay.contains(w))
}

/// Vrai si le lieu demandé est contenu dans le lieu de l'offre — ou si
/// l'offre est distante (toujours éligible).
fn location_matches(location: Option<&str>, job_location: &str, remote: bool) -> bool {
    match location.map(|l| l.trim().to_lowercase()) {
        Some(l) if !l.is_empty() => job_location.to_lowercase().contains(&l) || remote,
        _ => true,
    }
}

/// Formate une fourchette de salaire (« 40000-60000 EUR ») si pertinente.
fn salary_range(min: Option<&Value>, max: Option<&Value>, currency: &str) -> Option<String> {
    let mn = min.and_then(|v| v.as_i64()).filter(|v| *v > 0);
    let mx = max.and_then(|v| v.as_i64()).filter(|v| *v > 0);
    let cur = currency.trim();
    match (mn, mx) {
        (Some(a), Some(b)) if a == b => Some(format!("{a} {cur}").trim().to_string()),
        (Some(a), Some(b)) => Some(format!("{a}-{b} {cur}").trim().to_string()),
        (Some(a), None) => Some(format!("{a}+ {cur}").trim().to_string()),
        _ => None,
    }
}

/// Première valeur texte d'un tableau JSON (ex. `jobType: ["Full-Time"]`).
fn first_str(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|x| x.as_array())
        .and_then(|a| a.first())
        .and_then(|x| x.as_str())
        .map(String::from)
}

/// Toutes les valeurs texte d'un tableau JSON.
fn str_array(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

fn s(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").trim().to_string()
}

fn opt(s: String) -> Option<String> {
    if s.trim().is_empty() { None } else { Some(s) }
}

/// Retire rapidement les balises HTML d'une description.
pub fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Normalise les entités courantes et les espaces.
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
