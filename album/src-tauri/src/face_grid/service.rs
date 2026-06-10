use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::core::errors::{AppError, Result};
use crate::core::r2::R2Client;
use crate::db::repositories::face_grid_repo::{FaceGridRepository, FaceGridRow, FaceGridSlotRow};

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BdoAccount {
    pub account_id: String,
    pub characters: Vec<CharacterEntry>,
}

#[derive(Debug, Serialize)]
pub struct CharacterEntry {
    pub character_no: String,
    pub order:        u32,
    pub has_bmp:      bool,
    pub bmp_path:     Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FaceTextureEntry {
    pub character_no: String,
    pub path:         String,
}

#[derive(Debug, Deserialize)]
pub struct SlotAssignment {
    pub character_no: String,
    pub preset_id:    String,
    pub slot_order:   i32,
    pub image_url:    String,
}

// ── Paths ────────────────────────────────────────────────────────────────────

fn bdo_documents_path() -> PathBuf {
    let profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".into());
    PathBuf::from(profile).join("Documents").join("Black Desert")
}

fn user_cache_path() -> PathBuf {
    bdo_documents_path().join("UserCache")
}

fn face_texture_path() -> PathBuf {
    bdo_documents_path().join("FaceTexture")
}

// ── Service ──────────────────────────────────────────────────────────────────

pub struct FaceGridService;

impl FaceGridService {
    /// Scans all UserCache subdirectories for gamevariable.xml files.
    /// Each directory that contains CharacterOrderList = one BDO account.
    pub fn scan_bdo_accounts() -> Result<Vec<BdoAccount>> {
        let cache_dir = user_cache_path();
        let face_dir = face_texture_path();

        let mut accounts: Vec<BdoAccount> = Vec::new();

        let entries = std::fs::read_dir(&cache_dir).map_err(|e| {
            AppError::Io(e)
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let xml_path = path.join("gamevariable.xml");
            if !xml_path.exists() {
                continue;
            }

            let account_id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            match parse_character_order(&xml_path, &face_dir) {
                Ok(mut characters) => {
                    characters.sort_by_key(|c| c.order);
                    accounts.push(BdoAccount { account_id, characters });
                }
                Err(_) => {}
            }
        }

        accounts.sort_by(|a, b| a.account_id.cmp(&b.account_id));
        Ok(accounts)
    }

    /// Lists all .bmp files in FaceTexture, returning character_no + full path.
    pub fn list_face_textures() -> Result<Vec<FaceTextureEntry>> {
        let face_dir = face_texture_path();
        let mut entries: Vec<FaceTextureEntry> = Vec::new();

        for entry in std::fs::read_dir(&face_dir)?.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("bmp") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                entries.push(FaceTextureEntry {
                    character_no: stem.to_string(),
                    path:         path.to_string_lossy().into_owned(),
                });
            }
        }

        Ok(entries)
    }

    /// Downloads image_url, converts to BMP 24-bit, saves to FaceTexture/{character_no}.bmp.
    pub async fn apply_face_to_slot(character_no: &str, image_url: &str) -> Result<()> {
        let bytes = reqwest::get(image_url)
            .await?
            .bytes()
            .await
            .map_err(|e| AppError::Http(e.to_string()))?;

        let img = image::load_from_memory(&bytes)
            .map_err(|e| AppError::Internal(format!("image decode: {}", e)))?;

        let dest = face_texture_path().join(format!("{}.bmp", character_no));
        img.to_rgb8()
            .save(&dest)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        Ok(())
    }

    /// Saves a named face grid to the DB. Optionally uploads a thumbnail to R2.
    pub async fn save_face_grid(
        pool:       &PgPool,
        r2_client:  Option<&R2Client>,
        r2_pub:     &str,
        name:       &str,
        account_id: &str,
        slots:      &[SlotAssignment],
    ) -> Result<FaceGridRow> {
        // Create the grid record first (thumbnail_url updated after upload if needed)
        let grid = FaceGridRepository::create(pool, name, account_id, None).await?;

        // Upload thumbnail from the first slot's image (slot_order 0)
        let mut thumbnail_url: Option<String> = None;
        if let Some(r2) = r2_client {
            if let Some(first) = slots.iter().min_by_key(|s| s.slot_order) {
                if !first.image_url.is_empty() {
                    match Self::fetch_and_upload_thumb(r2, grid.id, &first.image_url).await {
                        Ok(url) => thumbnail_url = Some(url),
                        Err(_)  => {},
                    }
                }
            }
        }

        // Update thumbnail_url if we got one
        if let Some(ref url) = thumbnail_url {
            sqlx::query!(
                "UPDATE album_face_grids SET thumbnail_url = $1 WHERE id = $2",
                url, grid.id
            )
            .execute(pool)
            .await?;
        }

        // Upsert all slots
        for slot in slots {
            let pid: Option<i64> = slot.preset_id.parse().ok();
            FaceGridRepository::upsert_slot(pool, grid.id, &slot.character_no, pid, slot.slot_order).await?;
        }

        // Return fresh row
        let updated = FaceGridRepository::get_all(pool).await?;
        updated.into_iter().find(|g| g.id == grid.id)
            .ok_or_else(|| AppError::NotFound(format!("grid {}", grid.id)))
    }

    async fn fetch_and_upload_thumb(r2: &R2Client, grid_id: i64, image_url: &str) -> Result<String> {
        let bytes = reqwest::get(image_url)
            .await?
            .bytes()
            .await
            .map_err(|e| AppError::Http(e.to_string()))?;

        let img = image::load_from_memory(&bytes)
            .map_err(|e| AppError::Internal(format!("thumb decode: {}", e)))?;

        let thumb = img.thumbnail(256, 256);
        let mut buf: Vec<u8> = Vec::new();
        thumb.write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::WebP,
        ).map_err(|e| AppError::Internal(format!("thumb encode: {}", e)))?;

        let key = format!("face-grids/{}/thumb.webp", grid_id);
        r2.upload(&key, buf).await
    }

    pub async fn get_face_grids(pool: &PgPool) -> Result<Vec<FaceGridRow>> {
        FaceGridRepository::get_all(pool).await
    }

    pub async fn get_face_grid_slots(
        pool:   &PgPool,
        grid_id: i64,
        r2_pub:  &str,
    ) -> Result<Vec<FaceGridSlotRow>> {
        FaceGridRepository::get_slots(pool, grid_id, r2_pub).await
    }

    /// Applies all slots of a saved grid: downloads images and writes BMPs to FaceTexture.
    pub async fn apply_face_grid(
        pool:    &PgPool,
        grid_id: i64,
        r2_pub:  &str,
    ) -> Result<()> {
        let slots = FaceGridRepository::get_slots(pool, grid_id, r2_pub).await?;
        for slot in slots {
            if let Some(url) = slot.image_1_url {
                Self::apply_face_to_slot(&slot.character_no, &url).await.ok();
            }
        }
        Ok(())
    }

    pub async fn delete_face_grid(pool: &PgPool, grid_id: i64) -> Result<()> {
        FaceGridRepository::delete(pool, grid_id).await
    }
}

// ── XML parsing ──────────────────────────────────────────────────────────────

fn parse_character_order(xml_path: &PathBuf, face_dir: &PathBuf) -> Result<Vec<CharacterEntry>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let content = std::fs::read_to_string(xml_path)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut characters: Vec<CharacterEntry> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"CharacterOrderList" {
                    let mut char_no: Option<String> = None;
                    let mut order:   Option<u32>    = None;

                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let val = attr.unescape_value().unwrap_or_default().to_string();
                        match key {
                            "CharacterNo" => char_no = Some(val),
                            "Order"       => order   = val.parse().ok(),
                            _             => {}
                        }
                    }

                    if let (Some(character_no), Some(order)) = (char_no, order) {
                        let bmp_path = face_dir.join(format!("{}.bmp", character_no));
                        let has_bmp  = bmp_path.exists();
                        characters.push(CharacterEntry {
                            bmp_path: if has_bmp {
                                Some(bmp_path.to_string_lossy().into_owned())
                            } else {
                                None
                            },
                            character_no,
                            order,
                            has_bmp,
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(AppError::Xml(format!("{}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(characters)
}
