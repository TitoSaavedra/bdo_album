use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tauri::{AppHandle, Emitter};

use crate::core::errors::{AppError, Result};
use crate::core::r2::R2Client;
use crate::db::repositories::face_grid_repo::{FaceGridRepository, FaceGridRow, FaceGridSlotRow};

#[derive(Clone, Serialize)]
pub struct FaceGridProgress {
    pub current:      usize,
    pub total:        usize,
    pub character_no: String,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct SlotAssignment {
    pub character_no: String,
    pub slot_order:   i32,
    pub image_url:    String,
}

// ── Paths ────────────────────────────────────────────────────────────────────

fn bdo_documents_path() -> PathBuf {
    // dirs::document_dir() resolves the real Windows known-folder location,
    // which OneDrive redirects away from "%USERPROFILE%\Documents".
    let documents = dirs::document_dir().unwrap_or_else(|| {
        let profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".into());
        PathBuf::from(profile).join("Documents")
    });
    documents.join("Black Desert")
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

        let entries = match std::fs::read_dir(&cache_dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(accounts),
            Err(e) => return Err(AppError::Io(e)),
        };

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

        let dir_entries = match std::fs::read_dir(&face_dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(entries),
            Err(e) => return Err(AppError::Io(e)),
        };

        for entry in dir_entries.flatten() {
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

    /// Saves a named face grid to the DB. If slots are empty, reads BMPs from disk and uploads to R2.
    pub async fn save_face_grid(
        pool:       &PgPool,
        r2_client:  Option<&R2Client>,
        name:       &str,
        account_id: &str,
        slots:      &[SlotAssignment],
        app:        &AppHandle,
    ) -> Result<FaceGridRow> {
        // If slots are empty, prepare them from BMPs
        let mut final_slots = if slots.is_empty() {
            Self::prepare_slots_from_bmps(account_id).await?
        } else {
            slots.to_vec()
        };

        // Create the grid record first
        let grid = FaceGridRepository::create(pool, name, account_id, None).await?;

        // Upload BMPs to R2 for any slot with an empty image_url
        let needs_upload: Vec<usize> = final_slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.image_url.is_empty())
            .map(|(i, _)| i)
            .collect();

        if !needs_upload.is_empty() {
            let face_dir = face_texture_path();
            let total    = needs_upload.len();
            for (progress, &slot_idx) in needs_upload.iter().enumerate() {
                let slot = &mut final_slots[slot_idx];
                app.emit("face_grid_progress", FaceGridProgress {
                    current:      progress,
                    total,
                    character_no: slot.character_no.clone(),
                }).ok();

                let bmp_path = face_dir.join(format!("{}.bmp", slot.character_no));
                if bmp_path.exists() {
                    let bytes = std::fs::read(&bmp_path).map_err(AppError::Io)?;
                    let url = Self::process_and_store_face(
                        r2_client,
                        &slot.character_no,
                        &bytes,
                        Some(account_id),
                        Some(grid.id),
                    ).await.unwrap_or_default();
                    slot.image_url = url;
                }
            }
            app.emit("face_grid_progress", FaceGridProgress {
                current:      total,
                total,
                character_no: String::new(),
            }).ok();
        }

        // Upload thumbnail from the first slot's image (slot_order 0)
        let mut thumbnail_url: Option<String> = None;
        if let Some(r2) = r2_client {
            if let Some(first) = final_slots.iter().min_by_key(|s| s.slot_order) {
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
        for slot in &final_slots {
            FaceGridRepository::upsert_slot(pool, grid.id, &slot.character_no, &slot.image_url, slot.slot_order).await?;
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
        let _ = r2.upload(&key, buf).await?;
        Ok(key)
    }

    /// Overwrites an existing grid: clears old slots and re-uploads all BMPs from disk.
    pub async fn overwrite_face_grid(
        pool:       &PgPool,
        r2_client:  Option<&R2Client>,
        grid_id:    i64,
        account_id: &str,
        app:        &AppHandle,
    ) -> Result<FaceGridRow> {
        FaceGridRepository::clear_slots(pool, grid_id).await?;

        let mut slots = Self::prepare_slots_from_bmps(account_id).await?;

        let face_dir = face_texture_path();
        let total    = slots.len();
        for (i, slot) in slots.iter_mut().enumerate() {
            app.emit("face_grid_progress", FaceGridProgress {
                current:      i,
                total,
                character_no: slot.character_no.clone(),
            }).ok();

            let bmp_path = face_dir.join(format!("{}.bmp", slot.character_no));
            if bmp_path.exists() {
                let bytes = std::fs::read(&bmp_path).map_err(AppError::Io)?;
                let url = Self::process_and_store_face(
                    r2_client,
                    &slot.character_no,
                    &bytes,
                    Some(account_id),
                    Some(grid_id),
                ).await.unwrap_or_default();
                slot.image_url = url;
            }
        }
        app.emit("face_grid_progress", FaceGridProgress {
            current:      total,
            total,
            character_no: String::new(),
        }).ok();

        for slot in &slots {
            FaceGridRepository::upsert_slot(pool, grid_id, &slot.character_no, &slot.image_url, slot.slot_order).await?;
        }

        let updated = FaceGridRepository::get_all(pool).await?;
        updated.into_iter().find(|g| g.id == grid_id)
            .ok_or_else(|| AppError::NotFound(format!("grid {}", grid_id)))
    }

    pub async fn get_face_grids(pool: &PgPool) -> Result<Vec<FaceGridRow>> {
        FaceGridRepository::get_all(pool).await
    }

    pub async fn get_face_grid_slots(pool: &PgPool, grid_id: i64, r2_public_url: &str) -> Result<Vec<FaceGridSlotRow>> {
        FaceGridRepository::get_slots(pool, grid_id, r2_public_url).await
    }

    /// Applies all slots of a saved grid: downloads images and writes BMPs to FaceTexture.
    /// Emits `face_apply_done` after each character is written to disk.
    pub async fn apply_face_grid(pool: &PgPool, grid_id: i64, r2_public_url: &str, app: &AppHandle) -> Result<()> {
        let slots = FaceGridRepository::get_slots(pool, grid_id, r2_public_url).await?;
        for slot in slots {
            if !slot.image_url.is_empty() {
                Self::apply_face_to_slot(&slot.character_no, &slot.image_url).await.ok();
                app.emit("face_apply_done", &slot.character_no).ok();
            }
        }
        Ok(())
    }

    pub async fn delete_face_grid(pool: &PgPool, grid_id: i64) -> Result<()> {
        FaceGridRepository::delete(pool, grid_id).await
    }

    /// Helper: Process image (validate, convert to BMP, upload to R2)
    /// If account_id and grid_id are provided, use organized structure.
    /// Otherwise, use temp location for drag & drop.
    async fn process_and_store_face(
        r2_client:    Option<&R2Client>,
        character_no: &str,
        image_bytes:  &[u8],
        account_id:   Option<&str>,
        grid_id:      Option<i64>,
    ) -> Result<String> {
        // Load and validate image
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| AppError::Internal(format!("invalid image: {}", e)))?;
        let resized = img.thumbnail(512, 512);

        // Save to disk as BMP
        let face_dir = face_texture_path();
        let disk_path = face_dir.join(format!("{}.bmp", character_no));
        resized.to_rgb8()
            .save_with_format(&disk_path, image::ImageFormat::Bmp)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // Upload to R2 as WebP with organized path
        let key = if let (Some(acc_id), Some(gid)) = (account_id, grid_id) {
            format!("face-grids/{}/{}/{}.webp", acc_id, gid, character_no)
        } else {
            // Temp location for drag & drop
            format!("character-faces/{}.webp", character_no)
        };

        if let Some(r2) = r2_client {
            let mut buf: Vec<u8> = Vec::new();
            resized.write_to(
                &mut std::io::Cursor::new(&mut buf),
                image::ImageFormat::WebP,
            ).map_err(|e| AppError::Internal(format!("image encode: {}", e)))?;
            let _ = r2.upload(&key, buf).await?;
        }

        Ok(key)
    }

    pub async fn get_character_faces(pool: &PgPool) -> Result<Vec<(String, String)>> {
        FaceGridRepository::get_all_character_faces(pool).await
    }

    /// Drag & drop: read image from file_path, convert to BMP, save to disk. No R2 upload.
    pub async fn save_face_to_disk(character_no: &str, file_path: &str) -> Result<()> {
        let bytes = std::fs::read(file_path).map_err(AppError::Io)?;

        let img = image::load_from_memory(&bytes)
            .map_err(|e| AppError::Internal(format!("invalid image: {}", e)))?;

        let disk_path = face_texture_path().join(format!("{}.bmp", character_no));
        img.thumbnail(512, 512)
            .to_rgb8()
            .save_with_format(&disk_path, image::ImageFormat::Bmp)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        Ok(())
    }

    /// Prepares slots from BMPs on disk (without uploading to R2 yet).
    async fn prepare_slots_from_bmps(
        account_id: &str,
    ) -> Result<Vec<SlotAssignment>> {
        let accounts = Self::scan_bdo_accounts()?;
        let account = accounts
            .into_iter()
            .find(|a| a.account_id == account_id)
            .ok_or_else(|| AppError::NotFound(format!("account {} not found", account_id)))?;

        let mut slots = Vec::new();

        for char_entry in account.characters {
            slots.push(SlotAssignment {
                character_no: char_entry.character_no,
                slot_order:   char_entry.order as i32,
                image_url:    String::new(),
            });
        }

        Ok(slots)
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
