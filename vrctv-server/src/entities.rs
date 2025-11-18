#![allow(dead_code)]

use rusqlite::Connection;

pub struct TwitchUser {
    pub id: i64,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

impl TwitchUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            joined_at: chrono::Utc::now(),
        }
    }
    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Self>> {
        let mut stmt = conn.prepare("SELECT id, joined_at FROM twitch_users WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: row.get(0)?,
                joined_at: chrono::DateTime::from_timestamp_millis(row.get(1)?).unwrap(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&self, conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO twitch_users (id, joined_at) VALUES (?1, ?2)",
            (self.id, self.joined_at.timestamp_millis()),
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
        conn.execute("DELETE FROM twitch_users WHERE id = ?1", [id])?;
        Ok(())
    }
}

pub struct StreamlabsUser {
    pub id: i64,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

impl StreamlabsUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            joined_at: chrono::Utc::now(),
        }
    }
    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Self>> {
        let mut stmt = conn.prepare("SELECT id, joined_at FROM streamlabs_users WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: row.get(0)?,
                joined_at: chrono::DateTime::from_timestamp_millis(row.get(1)?).unwrap(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&self, conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO streamlabs_users (id, joined_at) VALUES (?1, ?2)",
            (self.id, self.joined_at.timestamp_millis()),
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
        conn.execute("DELETE FROM streamlabs_users WHERE id = ?1", [id])?;
        Ok(())
    }
}

pub struct ActiveKey {
    pub state: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ActiveKey {
    pub fn new(state: String) -> Self {
        Self {
            state,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn get(conn: &Connection, state: &str) -> rusqlite::Result<Option<Self>> {
        let mut stmt =
            conn.prepare("SELECT state, created_at FROM active_keys WHERE state = ?1")?;
        let mut rows = stmt.query([state])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                state: row.get(0)?,
                created_at: chrono::DateTime::from_timestamp_millis(row.get(1)?).unwrap(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&self, conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO active_keys (state, created_at) VALUES (?1, ?2)",
            (self.state.clone(), self.created_at.timestamp_millis()),
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, state: &str) -> rusqlite::Result<()> {
        conn.execute("DELETE FROM active_keys WHERE state = ?1", [state])?;
        Ok(())
    }
}

pub struct ActiveTwitchKey {
    pub id: i64,
    pub authentication: String,
    pub refresh: String,
    pub user: i64,
    pub state: String,
    pub version: i64,
}

impl ActiveTwitchKey {
    pub fn new(
        authentication: String,
        refresh: String,
        user: i64,
        state: String,
        version: i64,
    ) -> Self {
        Self {
            id: 0,
            authentication,
            refresh,
            user,
            state,
            version,
        }
    }

    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Self>> {
        let mut stmt = conn.prepare("SELECT id, authentication, refresh, user, state, version FROM active_twitch_keys WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: row.get(0)?,
                authentication: row.get(1)?,
                refresh: row.get(2)?,
                user: row.get(3)?,
                state: row.get(4)?,
                version: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_active_key(conn: &Connection, state: &str) -> rusqlite::Result<Option<Self>> {
        let mut stmt = conn.prepare("SELECT id, authentication, refresh, user, state, version FROM active_twitch_keys WHERE state = ?1")?;
        let mut rows = stmt.query([state])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: row.get(0)?,
                authentication: row.get(1)?,
                refresh: row.get(2)?,
                user: row.get(3)?,
                state: row.get(4)?,
                version: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&mut self, conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO active_twitch_keys (authentication, refresh, user, state, version) VALUES (?1, ?2, ?3, ?4, ?5)",
            (self.authentication.clone(), self.refresh.clone(), self.user, self.state.clone(), self.version),
        )?;
        self.id = conn.last_insert_rowid();
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
        conn.execute("DELETE FROM active_twitch_keys WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update(&self, conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE active_twitch_keys SET authentication = ?1, refresh = ?2, user = ?3, state = ?4, version = ?5 WHERE id = ?6",
            (self.authentication.clone(), self.refresh.clone(), self.user, self.state.clone(), self.version, self.id),
        )?;
        Ok(())
    }
}

pub struct ActiveStreamLabsKey {
    pub id: i64,
    pub authentication: String,
    pub refresh: String,
    pub user: i64,
    pub state: String,
    pub version: i64,
}

impl ActiveStreamLabsKey {
    pub fn new(
        authentication: String,
        refresh: String,
        user: i64,
        state: String,
        version: i64,
    ) -> Self {
        Self {
            id: 0,
            authentication,
            refresh,
            user,
            state,
            version,
        }
    }

    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Self>> {
        let mut stmt = conn.prepare("SELECT id, authentication, refresh, user, state, version FROM active_stream_labs_keys WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: row.get(0)?,
                authentication: row.get(1)?,
                refresh: row.get(2)?,
                user: row.get(3)?,
                state: row.get(4)?,
                version: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_active_key(conn: &Connection, state: &str) -> rusqlite::Result<Option<Self>> {
        let mut stmt = conn.prepare("SELECT id, authentication, refresh, user, state, version FROM active_stream_labs_keys WHERE state = ?1")?;
        let mut rows = stmt.query([state])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: row.get(0)?,
                authentication: row.get(1)?,
                refresh: row.get(2)?,
                user: row.get(3)?,
                state: row.get(4)?,
                version: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&mut self, conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO active_stream_labs_keys (authentication, refresh, user, state, version) VALUES (?1, ?2, ?3, ?4, ?5)",
            (self.authentication.clone(), self.refresh.clone(), self.user, self.state.clone(), self.version),
        )?;
        self.id = conn.last_insert_rowid();
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
        conn.execute("DELETE FROM active_stream_labs_keys WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update(&self, conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE active_stream_labs_keys SET authentication = ?1, refresh = ?2, user = ?3, state = ?4, version = ?5 WHERE id = ?6",
            (self.authentication.clone(), self.refresh.clone(), self.user, self.state.clone(), self.version, self.id),
        )?;
        Ok(())
    }
}
