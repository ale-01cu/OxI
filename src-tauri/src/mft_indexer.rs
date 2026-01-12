use crate::db::Database;
use crate::types::{FileRecord, IndexingProgress};
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::Utc;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{info, warn};

const SECTOR_SIZE: usize = 512;
const MFT_RECORD_SIZE: usize = 1024;
const ATTR_FILENAME: u32 = 0x30;
const END_OF_ATTRIBUTES: u32 = 0xFFFFFFFF;

pub struct MftIndexer {
    db: Arc<Mutex<Database>>,
}

impl MftIndexer {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    pub async fn index_drive(
        &self,
        drive: &str,
        progress_callback: Arc<dyn Fn(IndexingProgress) + Send + Sync>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        info!("Starting MFT indexing of drive: {}", drive);
        let start = Instant::now();

        let path = format!(r"\\.\{}:", drive);
        let f = File::open(&path).map_err(|e| {
            warn!("Failed to open drive {} for MFT access: {}", drive, e);
            e
        })?;

        let mut reader = SectorReader::new(f, 4096);

        let mut boot_sector = vec![0u8; 512];
        reader.seek(SeekFrom::Start(0))?;
        reader.read_exact(&mut boot_sector)?;

        let mut cursor = Cursor::new(&boot_sector);
        cursor.set_position(0x0B);
        let bytes_per_sector = cursor.read_u16::<LittleEndian>()? as u64;
        cursor.set_position(0x0D);
        let sectors_per_cluster = cursor.read_u8()? as u64;
        cursor.set_position(0x30);
        let mft_cluster_lcn = cursor.read_u64::<LittleEndian>()?;

        let cluster_size = bytes_per_sector * sectors_per_cluster;
        let mft_offset = mft_cluster_lcn * cluster_size;

        info!(
            "MFT geometry: Sector={} Cluster={} MFT_LCN={} Offset={}",
            bytes_per_sector, cluster_size, mft_cluster_lcn, mft_offset
        );

        reader.seek(SeekFrom::Start(mft_offset))?;

        let mut records_processed = 0;
        let mut files_found = 0;
        let mut buffer = vec![0u8; MFT_RECORD_SIZE];
        const BATCH_SIZE: usize = 5_000;
        let mut batch_buffer: Vec<FileRecord> = Vec::with_capacity(BATCH_SIZE);

        let max_scan = 1_000_000;

        for i in 0..max_scan {
            if reader.read_exact(&mut buffer).is_err() {
                break;
            }
            records_processed += 1;

            if &buffer[0..4] != b"FILE" {
                continue;
            }

            if !apply_fixups(&mut buffer, bytes_per_sector as usize) {
                continue;
            }

            let mut rdr = Cursor::new(&buffer);
            rdr.set_position(0x16);
            let flags = rdr.read_u16::<LittleEndian>()?;
            let in_use = (flags & 0x01) != 0;

            rdr.set_position(0x14);
            let first_attr_offset = rdr.read_u16::<LittleEndian>()? as u64;
            rdr.set_position(first_attr_offset);

            let mut filename = None;
            let mut file_size = None;
            let mut is_dir = false;

            loop {
                if rdr.position() >= MFT_RECORD_SIZE as u64 - 8 {
                    break;
                }
                let attr_start_pos = rdr.position();
                let attr_type = rdr.read_u32::<LittleEndian>()?;
                if attr_type == END_OF_ATTRIBUTES {
                    break;
                }
                let attr_len = rdr.read_u32::<LittleEndian>()?;
                if attr_len == 0 {
                    break;
                }

                if attr_type == ATTR_FILENAME && filename.is_none() {
                    rdr.set_position(attr_start_pos + 8);
                    let non_resident = rdr.read_u8()? != 0;

                    if !non_resident {
                        rdr.set_position(attr_start_pos + 20);
                        let content_offset = rdr.read_u16::<LittleEndian>()? as u64;
                        let absolute_content_pos = attr_start_pos + content_offset;
                        rdr.set_position(absolute_content_pos);

                        if rdr.seek(SeekFrom::Current(48)).is_ok() {
                            let flags = rdr.read_u32::<LittleEndian>()?;
                            is_dir = (flags & 0x10000000) != 0;
                        }

                        if rdr.seek(SeekFrom::Current(8)).is_ok() {
                            let size = rdr.read_u64::<LittleEndian>()?;
                            file_size = Some(size as i64);
                        }

                        rdr.set_position(absolute_content_pos);

                        if rdr.seek(SeekFrom::Current(64)).is_err() {
                            rdr.set_position(attr_start_pos + attr_len as u64);
                            continue;
                        }

                        let name_len = rdr.read_u8()?;
                        let _namespace = rdr.read_u8()?;

                        let name_bytes_len = (name_len as usize) * 2;
                        let mut name_buffer = vec![0u8; name_bytes_len];

                        if rdr.read_exact(&mut name_buffer).is_ok() {
                            let u16_vec: Vec<u16> = name_buffer
                                .chunks_exact(2)
                                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                                .collect();

                            if let Ok(name) = String::from_utf16(&u16_vec) {
                                filename = Some(name);
                            }
                        }
                    }
                }

                rdr.set_position(attr_start_pos + attr_len as u64);
            }

            if let Some(name) = filename {
                if in_use && !name.is_empty() {
                    let path = format!("{}:\\{}", drive, name);
                    let modified_time_str = Utc::now().to_rfc3339();
                    let last_indexed_str = Utc::now().to_rfc3339();

                    let extension = if is_dir {
                        None
                    } else {
                        name.rfind('.').map(|idx| format!(".{}", &name[idx..]))
                    };

                    batch_buffer.push(FileRecord {
                        path,
                        name,
                        extension,
                        file_size,
                        is_dir,
                        modified_time: modified_time_str,
                        last_indexed: last_indexed_str,
                    });

                    files_found += 1;

                    progress_callback(IndexingProgress {
                        current_path: format!("{}\\...", drive),
                        files_processed: files_found,
                        total_files: None,
                        status: "indexing".to_string(),
                    });

                    if batch_buffer.len() >= BATCH_SIZE {
                        self.flush_batch(&mut batch_buffer)?;
                    }
                }
            }

            if i % 50000 == 0 && i > 0 {
                info!("MFT Progress: {} records analyzed...", i);
            }
        }

        self.flush_batch(&mut batch_buffer)?;

        let elapsed = start.elapsed();
        info!(
            "MFT indexing completed: processed={} files_found={} in {:?}",
            records_processed, files_found, elapsed
        );

        Ok(files_found)
    }

    fn flush_batch(&self, batch: &mut Vec<FileRecord>) -> Result<usize, Box<dyn std::error::Error>> {
        if batch.is_empty() {
            return Ok(0);
        }

        let mut db_guard = self.db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
        let batch_len = batch.len();

        match db_guard.upsert_batch(batch.as_slice()) {
            Ok(()) => {
                batch.clear();
                Ok(batch_len)
            }
            Err(e) => {
                warn!("Batch upsert failed ({} items): {}. Falling back to item-by-item.", batch_len, e);

                let mut ok_count = 0usize;
                for r in batch.iter() {
                    if let Err(item_err) = db_guard.upsert_file(
                        r.path.as_str(),
                        r.name.as_str(),
                        r.extension.as_deref(),
                        r.file_size,
                        r.is_dir,
                        r.modified_time.as_str(),
                        r.last_indexed.as_str(),
                    ) {
                        warn!("Failed to upsert {}: {}", r.path, item_err);
                    } else {
                        ok_count += 1;
                    }
                }

                batch.clear();
                Ok(ok_count)
            }
        }
    }
}

fn apply_fixups(buffer: &mut [u8], bytes_per_sector: usize) -> bool {
    if buffer.len() < 8 {
        return false;
    }

    let usa_offset = u16::from_le_bytes([buffer[4], buffer[5]]) as usize;
    let usa_count = u16::from_le_bytes([buffer[6], buffer[7]]) as usize;

    let start = usa_offset;
    let end = start + (usa_count * 2);

    if end > buffer.len() {
        return false;
    }

    let fixup_data = buffer[start..end].to_vec();
    let usn = u16::from_le_bytes([fixup_data[0], fixup_data[1]]);

    for i in 0..(usa_count - 1) {
        let idx = (i + 1) * 2;
        let original_bytes = u16::from_le_bytes([fixup_data[idx], fixup_data[idx + 1]]);
        let position_of_strip = (i + 1) * bytes_per_sector - 2;

        if position_of_strip + 2 > buffer.len() {
            return false;
        }

        let disk_bytes =
            u16::from_le_bytes([buffer[position_of_strip], buffer[position_of_strip + 1]]);

        if disk_bytes != usn {
            return false;
        }

        let bytes = original_bytes.to_le_bytes();
        buffer[position_of_strip] = bytes[0];
        buffer[position_of_strip + 1] = bytes[1];
    }

    true
}

struct SectorReader<R> {
    inner: R,
    sector_size: usize,
    buffer: Vec<u8>,
    buffer_offset: u64,
    buffer_valid: usize,
    pos: u64,
}

impl<R: Read + Seek> SectorReader<R> {
    pub fn new(inner: R, sector_size: usize) -> Self {
        let raw_vec = vec![0u8; sector_size * 2];
        Self {
            inner,
            sector_size,
            buffer: raw_vec,
            buffer_offset: 0,
            buffer_valid: 0,
            pos: 0,
        }
    }
}

impl<R: Read + Seek> Read for SectorReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let current_sector_start =
            (self.pos / self.sector_size as u64) * self.sector_size as u64;
        let offset_in_sector = (self.pos % self.sector_size as u64) as usize;

        if self.buffer_valid == 0 || self.buffer_offset != current_sector_start {
            self.inner.seek(SeekFrom::Start(current_sector_start))?;
            let ptr = self.buffer.as_ptr();
            let align_offset = ptr.align_offset(self.sector_size);
            let sector_slice =
                &mut self.buffer[align_offset..align_offset + self.sector_size];
            let read_bytes = self.inner.read(sector_slice)?;
            self.buffer_valid = read_bytes;
            self.buffer_offset = current_sector_start;
            if read_bytes == 0 {
                return Ok(0);
            }
        }

        let available_in_sector = self.buffer_valid.saturating_sub(offset_in_sector);
        let to_copy = std::cmp::min(buf.len(), available_in_sector);

        if to_copy == 0 && available_in_sector == 0 {
            self.buffer_valid = 0;
            return self.read(buf);
        }

        let ptr = self.buffer.as_ptr();
        let align_offset = ptr.align_offset(self.sector_size);
        let sector_slice = &self.buffer[align_offset..align_offset + self.sector_size];
        buf[..to_copy].copy_from_slice(
            &sector_slice[offset_in_sector..offset_in_sector + to_copy],
        );
        self.pos += to_copy as u64;
        Ok(to_copy)
    }
}

impl<R: Seek> Seek for SectorReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(v) => v,
            SeekFrom::End(v) => self.inner.seek(SeekFrom::End(v))?,
            SeekFrom::Current(v) => (self.pos as i64 + v) as u64,
        };
        self.pos = new_pos;
        Ok(self.pos)
    }
}
