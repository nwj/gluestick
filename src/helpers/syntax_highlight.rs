use crate::{db::Database, models::paste::Paste};
use rusqlite::{named_params, Transaction, TransactionBehavior};
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};
use uuid::Uuid;

pub fn generate(body: &str, extension: Option<&str>) -> Option<String> {
    let extension = extension?;

    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax = syntax_set.find_syntax_by_extension(extension)?;
    let theme = ThemeSet::get_theme("src/helpers/syntax_highlight_themes/CatppuccinFrappe.tmTheme")
        .map_err(|err| tracing::error!("failed to get syntax highlighting theme: {}", err))
        .ok()?;
    highlighted_html_for_string(body, &syntax_set, syntax, &theme).ok()
}

pub async fn generate_with_cache_attempt(
    db: &Database,
    paste_id: &Uuid,
    body: &str,
    extension: Option<&str>,
) -> tokio_rusqlite::Result<Option<String>> {
    if let Some(html) = cache_get(db, paste_id).await? {
        return Ok(Some(html));
    }

    // Why not do all of this as a single transaction? It's because the generate call is expensive
    // and we want to keep that work off the database thread and outside of SQL transactions. This
    // comes at the cost of an additional cache read and (in rare cases) potentially redundant html
    // generation.
    let optional_html = generate(body, extension);
    let paste_id = *paste_id;
    if let Some(html) = optional_html.clone() {
        db.conn
            .call(move |conn| {
                let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
                {
                    // We want to avoid race conditions that cache old incorrect html to the cache.
                    // The possible races are:
                    //
                    // - An update operation cached different html for the paste in the interim
                    // - A delete operation deleted the paste in the interim.
                    //
                    // So we check both in this transaction before writing.
                    if tx_cache_get(&tx, &paste_id)?.is_none()
                        && Paste::tx_find(&tx, &paste_id)?.is_some()
                    {
                        tx_cache_set(&tx, &paste_id, &html)?;
                    }
                }
                tx.commit()?;
                Ok(())
            })
            .await?;
    }

    Ok(optional_html)
}

pub async fn cache_get(db: &Database, paste_id: &Uuid) -> tokio_rusqlite::Result<Option<String>> {
    let paste_id = *paste_id;
    db.conn
        .call(move |conn| {
            let mut stmt = conn
                .prepare("SELECT html FROM syntax_highlight_cache WHERE paste_id = :paste_id;")?;
            let mut rows = stmt.query(named_params! {":paste_id": paste_id})?;
            match rows.next()? {
                Some(row) => Ok(Some(row.get::<usize, String>(0)?)),
                None => Ok(None),
            }
        })
        .await
}

pub fn tx_cache_get(tx: &Transaction, paste_id: &Uuid) -> rusqlite::Result<Option<String>> {
    let mut stmt =
        tx.prepare("SELECT html FROM syntax_highlight_cache WHERE paste_id = :paste_id;")?;
    let mut rows = stmt.query(named_params! {":paste_id": paste_id})?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get::<usize, String>(0)?)),
        None => Ok(None),
    }
}

pub fn tx_cache_set(tx: &Transaction, paste_id: &Uuid, html: &str) -> rusqlite::Result<()> {
    let mut stmt = tx.prepare("INSERT INTO syntax_highlight_cache VALUES (:paste_id, :html) ON CONFLICT DO UPDATE SET html = :html;")?;
    stmt.execute(named_params! {":paste_id": paste_id, ":html": html})?;
    Ok(())
}

pub fn tx_cache_expire(tx: &Transaction, paste_id: &Uuid) -> rusqlite::Result<()> {
    let mut stmt = tx.prepare("DELETE FROM syntax_highlight_cache WHERE paste_id = :paste_id;")?;
    stmt.execute(named_params! {":paste_id": paste_id})?;
    Ok(())
}
