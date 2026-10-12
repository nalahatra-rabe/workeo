use anyhow::Context;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;

use crate::models::{
    Application, Education, Experience, Job, Language, Match, Profile, SearchRun,
};

/// Initialise le pool SQLite (WAL) et crée le schéma.
pub async fn init_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    // Garantit que le dossier data/ existe pour un fichier sqlite://data/xxx.db
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        if !path.starts_with(':') {
            if let Some(parent) = std::path::Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).ok();
                }
            }
        }
    }

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(10));

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await
        .context("connexion SQLite")?;

    migrate(&pool).await?;
    Ok(pool)
}

async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    let schema = r#"
    CREATE TABLE IF NOT EXISTS profiles (
        id                TEXT PRIMARY KEY,
        full_name         TEXT NOT NULL,
        email             TEXT,
        phone             TEXT,
        location          TEXT,
        headline          TEXT,
        summary           TEXT,
        seniority         TEXT,
        years_experience  REAL,
        skills            TEXT NOT NULL DEFAULT '[]',
        languages         TEXT NOT NULL DEFAULT '[]',
        education         TEXT NOT NULL DEFAULT '[]',
        experiences       TEXT NOT NULL DEFAULT '[]',
        domains           TEXT NOT NULL DEFAULT '[]',
        target_roles      TEXT NOT NULL DEFAULT '[]',
        search_keywords   TEXT NOT NULL DEFAULT '[]',
        category          TEXT,
        remote_preference TEXT,
        contract_types    TEXT NOT NULL DEFAULT '[]',
        salary_min        INTEGER,
        cv_filename       TEXT,
        cv_text           TEXT,
        created_at        TEXT NOT NULL,
        updated_at        TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS jobs (
        id              TEXT PRIMARY KEY,
        source          TEXT NOT NULL,
        external_id     TEXT NOT NULL,
        title           TEXT NOT NULL,
        company         TEXT NOT NULL,
        location        TEXT,
        remote          INTEGER NOT NULL DEFAULT 0,
        description     TEXT,
        url             TEXT,
        summary         TEXT,
        salary_min      INTEGER,
        salary_max      INTEGER,
        salary_currency TEXT,
        contract_type   TEXT,
        seniority       TEXT,
        category        TEXT,
        skills          TEXT NOT NULL DEFAULT '[]',
        posted_at       TEXT,
        scraped_at      TEXT NOT NULL,
        UNIQUE(source, external_id)
    );
    CREATE INDEX IF NOT EXISTS idx_jobs_remote   ON jobs(remote);
    CREATE INDEX IF NOT EXISTS idx_jobs_category ON jobs(category);
    CREATE INDEX IF NOT EXISTS idx_jobs_scraped  ON jobs(scraped_at);

    CREATE TABLE IF NOT EXISTS matches (
        id         TEXT PRIMARY KEY,
        profile_id TEXT NOT NULL,
        job_id     TEXT NOT NULL,
        score      INTEGER NOT NULL,
        reasons    TEXT NOT NULL DEFAULT '[]',
        created_at TEXT NOT NULL,
        UNIQUE(profile_id, job_id)
    );
    CREATE INDEX IF NOT EXISTS idx_matches_profile ON matches(profile_id, score DESC);

    CREATE TABLE IF NOT EXISTS applications (
        id         TEXT PRIMARY KEY,
        profile_id TEXT NOT NULL,
        job_id     TEXT NOT NULL,
        status     TEXT NOT NULL DEFAULT 'saved',
        notes      TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        UNIQUE(profile_id, job_id)
    );

    CREATE TABLE IF NOT EXISTS search_runs (
        id          TEXT PRIMARY KEY,
        profile_id  TEXT NOT NULL,
        query       TEXT,
        location    TEXT,
        sources     TEXT NOT NULL DEFAULT '[]',
        status      TEXT NOT NULL DEFAULT 'running',
        jobs_found  INTEGER NOT NULL DEFAULT 0,
        jobs_new    INTEGER NOT NULL DEFAULT 0,
        started_at  TEXT NOT NULL,
        finished_at TEXT
    );
    "#;

    for stmt in schema.split(';') {
        let stmt = stmt.trim();
        if !stmt.is_empty() {
            sqlx::query(stmt).execute(pool).await?;
        }
    }
    Ok(())
}

// ─── Helpers JSON ───────────────────────────────────────────

fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "[]".into())
}
fn from_json<T: serde::de::DeserializeOwned + Default>(s: Option<&str>) -> T {
    s.and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default()
}

// ─── Profils ────────────────────────────────────────────────

pub fn row_to_profile(row: &sqlx::sqlite::SqliteRow) -> Profile {
    Profile {
        id: row.get("id"),
        full_name: row.get("full_name"),
        email: row.get("email"),
        phone: row.get("phone"),
        location: row.get("location"),
        headline: row.get("headline"),
        summary: row.get("summary"),
        seniority: row.get("seniority"),
        years_experience: row.get("years_experience"),
        skills: from_json(row.get::<Option<String>, _>("skills").as_deref()),
        languages: from_json::<Vec<Language>>(row.get::<Option<String>, _>("languages").as_deref()),
        education: from_json::<Vec<Education>>(row.get::<Option<String>, _>("education").as_deref()),
        experiences: from_json::<Vec<Experience>>(
            row.get::<Option<String>, _>("experiences").as_deref(),
        ),
        domains: from_json(row.get::<Option<String>, _>("domains").as_deref()),
        target_roles: from_json(row.get::<Option<String>, _>("target_roles").as_deref()),
        search_keywords: from_json(row.get::<Option<String>, _>("search_keywords").as_deref()),
        category: row.get("category"),
        remote_preference: row.get("remote_preference"),
        contract_types: from_json(row.get::<Option<String>, _>("contract_types").as_deref()),
        salary_min: row.get("salary_min"),
        cv_filename: row.get("cv_filename"),
        cv_text: row.get("cv_text"),
        created_at: parse_dt(row.get::<String, _>("created_at").as_str()),
        updated_at: parse_dt(row.get::<String, _>("updated_at").as_str()),
    }
}

pub async fn insert_profile(pool: &SqlitePool, p: &Profile) -> anyhow::Result<()> {
    sqlx::query(
        r#"INSERT INTO profiles
        (id, full_name, email, phone, location, headline, summary, seniority, years_experience,
         skills, languages, education, experiences, domains, target_roles, search_keywords,
         category, remote_preference, contract_types, salary_min, cv_filename, cv_text,
         created_at, updated_at)
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)"#,
    )
    .bind(&p.id)
    .bind(&p.full_name)
    .bind(&p.email)
    .bind(&p.phone)
    .bind(&p.location)
    .bind(&p.headline)
    .bind(&p.summary)
    .bind(&p.seniority)
    .bind(p.years_experience)
    .bind(to_json(&p.skills))
    .bind(to_json(&p.languages))
    .bind(to_json(&p.education))
    .bind(to_json(&p.experiences))
    .bind(to_json(&p.domains))
    .bind(to_json(&p.target_roles))
    .bind(to_json(&p.search_keywords))
    .bind(&p.category)
    .bind(&p.remote_preference)
    .bind(to_json(&p.contract_types))
    .bind(p.salary_min)
    .bind(&p.cv_filename)
    .bind(&p.cv_text)
    .bind(p.created_at.to_rfc3339())
    .bind(p.updated_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_profile(pool: &SqlitePool, p: &Profile) -> anyhow::Result<()> {
    sqlx::query(
        r#"UPDATE profiles SET
            full_name=?2, email=?3, phone=?4, location=?5, headline=?6, summary=?7,
            seniority=?8, years_experience=?9, skills=?10, languages=?11, education=?12,
            experiences=?13, domains=?14, target_roles=?15, search_keywords=?16, category=?17,
            remote_preference=?18, contract_types=?19, salary_min=?20, cv_filename=?21, cv_text=?22,
            updated_at=?23
        WHERE id=?1"#,
    )
    .bind(&p.id)
    .bind(&p.full_name)
    .bind(&p.email)
    .bind(&p.phone)
    .bind(&p.location)
    .bind(&p.headline)
    .bind(&p.summary)
    .bind(&p.seniority)
    .bind(p.years_experience)
    .bind(to_json(&p.skills))
    .bind(to_json(&p.languages))
    .bind(to_json(&p.education))
    .bind(to_json(&p.experiences))
    .bind(to_json(&p.domains))
    .bind(to_json(&p.target_roles))
    .bind(to_json(&p.search_keywords))
    .bind(&p.category)
    .bind(&p.remote_preference)
    .bind(to_json(&p.contract_types))
    .bind(p.salary_min)
    .bind(&p.cv_filename)
    .bind(&p.cv_text)
    .bind(p.updated_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_profile(pool: &SqlitePool, id: &str) -> anyhow::Result<Option<Profile>> {
    let row = sqlx::query("SELECT * FROM profiles WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(row_to_profile))
}

// ─── Offres ─────────────────────────────────────────────────

pub fn row_to_job(row: &sqlx::sqlite::SqliteRow) -> Job {
    Job {
        id: row.get("id"),
        source: row.get("source"),
        external_id: row.get("external_id"),
        title: row.get("title"),
        company: row.get("company"),
        location: row.get("location"),
        remote: row.get::<i64, _>("remote") != 0,
        description: row.get("description"),
        url: row.get("url"),
        summary: row.get("summary"),
        salary_min: row.get("salary_min"),
        salary_max: row.get("salary_max"),
        salary_currency: row.get("salary_currency"),
        contract_type: row.get("contract_type"),
        seniority: row.get("seniority"),
        category: row.get("category"),
        skills: from_json(row.get::<Option<String>, _>("skills").as_deref()),
        posted_at: row.get::<Option<String>, _>("posted_at").map(|s| parse_dt(&s)),
        scraped_at: parse_dt(row.get::<String, _>("scraped_at").as_str()),
    }
}

/// Insère ou met à jour une offre (upsert sur `(source, external_id)`).
/// Renvoie `true` si l'offre est nouvelle.
pub async fn upsert_job(pool: &SqlitePool, job: &Job) -> anyhow::Result<bool> {
    let existing: Option<String> =
        sqlx::query_scalar("SELECT id FROM jobs WHERE source=?1 AND external_id=?2")
            .bind(&job.source)
            .bind(&job.external_id)
            .fetch_optional(pool)
            .await?;

    let is_new = existing.is_none();
    sqlx::query(
        r#"INSERT INTO jobs
        (id, source, external_id, title, company, location, remote, description, url, summary,
         salary_min, salary_max, salary_currency, contract_type, seniority, category, skills,
         posted_at, scraped_at)
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)
        ON CONFLICT(source, external_id) DO UPDATE SET
            title=excluded.title, company=excluded.company, location=excluded.location,
            remote=excluded.remote, description=excluded.description, url=excluded.url,
            summary=excluded.summary, salary_min=excluded.salary_min, salary_max=excluded.salary_max,
            salary_currency=excluded.salary_currency, contract_type=excluded.contract_type,
            seniority=excluded.seniority, category=excluded.category, skills=excluded.skills,
            posted_at=excluded.posted_at, scraped_at=excluded.scraped_at"#,
    )
    .bind(&job.id)
    .bind(&job.source)
    .bind(&job.external_id)
    .bind(&job.title)
    .bind(&job.company)
    .bind(&job.location)
    .bind(job.remote as i64)
    .bind(&job.description)
    .bind(&job.url)
    .bind(&job.summary)
    .bind(job.salary_min)
    .bind(job.salary_max)
    .bind(&job.salary_currency)
    .bind(&job.contract_type)
    .bind(&job.seniority)
    .bind(&job.category)
    .bind(to_json(&job.skills))
    .bind(job.posted_at.map(|d| d.to_rfc3339()))
    .bind(job.scraped_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(is_new)
}

pub async fn get_job(pool: &SqlitePool, id: &str) -> anyhow::Result<Option<Job>> {
    let row = sqlx::query("SELECT * FROM jobs WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(row_to_job))
}

/// Retourne un lot d'offres à catégoriser, prioritairement les non catégorisées.
#[allow(dead_code)]
pub async fn jobs_for_categorization(pool: &SqlitePool, limit: i64) -> anyhow::Result<Vec<Job>> {
    let rows = sqlx::query(
        "SELECT * FROM jobs WHERE category IS NULL ORDER BY scraped_at DESC LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(row_to_job).collect())
}

// ─── Matchs ─────────────────────────────────────────────────

pub async fn upsert_match(
    pool: &SqlitePool,
    profile_id: &str,
    job_id: &str,
    score: i64,
    reasons: &[String],
) -> anyhow::Result<()> {
    sqlx::query(
        r#"INSERT INTO matches (id, profile_id, job_id, score, reasons, created_at)
           VALUES (?1,?2,?3,?4,?5,?6)
           ON CONFLICT(profile_id, job_id) DO UPDATE SET
               score=excluded.score, reasons=excluded.reasons, created_at=excluded.created_at"#,
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(profile_id)
    .bind(job_id)
    .bind(score)
    .bind(to_json(&reasons))
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub fn row_to_match(row: &sqlx::sqlite::SqliteRow) -> Match {
    Match {
        id: row.get("id"),
        profile_id: row.get("profile_id"),
        job_id: row.get("job_id"),
        score: row.get("score"),
        reasons: from_json(row.get::<Option<String>, _>("reasons").as_deref()),
        created_at: parse_dt(row.get::<String, _>("created_at").as_str()),
    }
}

pub async fn get_match(
    pool: &SqlitePool,
    profile_id: &str,
    job_id: &str,
) -> anyhow::Result<Option<Match>> {
    let row = sqlx::query("SELECT * FROM matches WHERE profile_id=?1 AND job_id=?2")
        .bind(profile_id)
        .bind(job_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(row_to_match))
}

// ─── Candidatures ───────────────────────────────────────────

pub fn row_to_application(row: &sqlx::sqlite::SqliteRow) -> Application {
    Application {
        id: row.get("id"),
        profile_id: row.get("profile_id"),
        job_id: row.get("job_id"),
        status: row.get("status"),
        notes: row.get("notes"),
        created_at: parse_dt(row.get::<String, _>("created_at").as_str()),
        updated_at: parse_dt(row.get::<String, _>("updated_at").as_str()),
    }
}

pub async fn upsert_application(
    pool: &SqlitePool,
    profile_id: &str,
    job_id: &str,
    status: &str,
    notes: Option<&str>,
) -> anyhow::Result<Application> {
    let now = Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO applications (id, profile_id, job_id, status, notes, created_at, updated_at)
           VALUES (?1,?2,?3,?4,?5,?6,?7)
           ON CONFLICT(profile_id, job_id) DO UPDATE SET status=excluded.status,
               notes=COALESCE(excluded.notes, applications.notes), updated_at=excluded.updated_at"#,
    )
    .bind(&id)
    .bind(profile_id)
    .bind(job_id)
    .bind(status)
    .bind(notes)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;
    let row = sqlx::query("SELECT * FROM applications WHERE profile_id=?1 AND job_id=?2")
        .bind(profile_id)
        .bind(job_id)
        .fetch_one(pool)
        .await?;
    Ok(row_to_application(&row))
}

// ─── Recherches ─────────────────────────────────────────────

pub fn row_to_search_run(row: &sqlx::sqlite::SqliteRow) -> SearchRun {
    SearchRun {
        id: row.get("id"),
        profile_id: row.get("profile_id"),
        query: row.get("query"),
        location: row.get("location"),
        sources: from_json(row.get::<Option<String>, _>("sources").as_deref()),
        status: row.get("status"),
        jobs_found: row.get("jobs_found"),
        jobs_new: row.get("jobs_new"),
        started_at: parse_dt(row.get::<String, _>("started_at").as_str()),
        finished_at: row.get::<Option<String>, _>("finished_at").map(|s| parse_dt(&s)),
    }
}

pub async fn insert_search_run(pool: &SqlitePool, run: &SearchRun) -> anyhow::Result<()> {
    sqlx::query(
        r#"INSERT INTO search_runs (id, profile_id, query, location, sources, status, jobs_found, jobs_new, started_at, finished_at)
           VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)"#,
    )
    .bind(&run.id)
    .bind(&run.profile_id)
    .bind(&run.query)
    .bind(&run.location)
    .bind(to_json(&run.sources))
    .bind(&run.status)
    .bind(run.jobs_found)
    .bind(run.jobs_new)
    .bind(run.started_at.to_rfc3339())
    .bind(run.finished_at.map(|d| d.to_rfc3339()))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn finish_search_run(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    found: i64,
    new: i64,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE search_runs SET status=?2, jobs_found=?3, jobs_new=?4, finished_at=?5 WHERE id=?1")
        .bind(id)
        .bind(status)
        .bind(found)
        .bind(new)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;
    Ok(())
}

// ─── Utilitaires ────────────────────────────────────────────

fn parse_dt(s: &str) -> DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

use chrono::DateTime;
