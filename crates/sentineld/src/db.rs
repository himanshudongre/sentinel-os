use rusqlite::{params, Connection, OptionalExtension};
use sentinel_core::ProofBundle;

#[derive(Clone)]
pub struct Db {
    path: String,
}

impl Db {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    fn conn(&self) -> rusqlite::Result<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Ok(conn)
    }

    pub fn init(&self) -> rusqlite::Result<()> {
        let conn = self.conn()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS proofs (
                seq           INTEGER PRIMARY KEY AUTOINCREMENT,
                proof_id      TEXT NOT NULL UNIQUE,
                ts            TEXT NOT NULL,
                log_hash      TEXT NOT NULL UNIQUE,
                prev_log_hash TEXT NOT NULL,
                pubkey_id     TEXT NOT NULL,
                bundle_json   TEXT NOT NULL,
                inserted_at   TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_proofs_seq ON proofs(seq);
            CREATE INDEX IF NOT EXISTS idx_proofs_prev ON proofs(prev_log_hash);
            CREATE INDEX IF NOT EXISTS idx_proofs_pubkey ON proofs(pubkey_id);
            ",
        )?;
        Ok(())
    }

    pub fn insert_proof(&self, proof: &ProofBundle) -> rusqlite::Result<()> {
        let conn = self.conn()?;
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

        let json = serde_json::to_string(proof).expect("proof serializable");

        conn.execute(
            "INSERT INTO proofs (proof_id, ts, log_hash, prev_log_hash, pubkey_id, bundle_json, inserted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                proof.proof_id.to_string(),
                proof.ts,
                proof.log_hash,
                proof.prev_log_hash,
                proof.signing.pubkey_id,
                json,
                now
            ],
        )?;
        Ok(())
    }

    pub fn get_proof_by_id(&self, proof_id: &str) -> rusqlite::Result<Option<ProofBundle>> {
        let conn = self.conn()?;
        conn.query_row(
            "SELECT bundle_json FROM proofs WHERE proof_id = ?1",
            params![proof_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map(|opt| opt.map(|s| serde_json::from_str(&s).expect("valid proof json")))
    }

    pub fn get_head(&self) -> rusqlite::Result<Option<ProofBundle>> {
        let conn = self.conn()?;
        conn.query_row(
            "SELECT bundle_json FROM proofs ORDER BY seq DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map(|opt| opt.map(|s| serde_json::from_str(&s).expect("valid proof json")))
    }

    pub fn list_chain(&self, limit: usize) -> rusqlite::Result<Vec<ProofBundle>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT bundle_json FROM proofs ORDER BY seq ASC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit as i64], |row| row.get::<_, String>(0))?;

        let mut out = Vec::new();
        for r in rows {
            let s = r?;
            out.push(serde_json::from_str(&s).expect("valid proof json"));
        }
        Ok(out)
    }

    pub fn expected_prev_log_hash(&self) -> rusqlite::Result<String> {
        match self.get_head()? {
            Some(p) => Ok(p.log_hash),
            None => Ok("00".repeat(32)),
        }
    }
}
