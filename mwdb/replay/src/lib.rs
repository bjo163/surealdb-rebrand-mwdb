use mwdb_local_first::{LocalFirstError, LocalFirstStore, LogicalChangeV1};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("local-first error: {0}")]
    LocalFirst(#[from] LocalFirstError),
    #[error("invalid logical change at line {line}: {source}")]
    InvalidLine {
        line: usize,
        #[source]
        source: serde_json::Error,
    },
}

pub fn import_jsonl(store: &mut LocalFirstStore, jsonl: &str) -> Result<usize, ReplayError> {
    let mut imported = 0;
    for (index, line) in jsonl.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let change: LogicalChangeV1 = serde_json::from_str(line).map_err(|source| ReplayError::InvalidLine {
            line: index + 1,
            source,
        })?;
        if store.apply_remote(&change)? {
            imported += 1;
        }
    }
    Ok(imported)
}

pub fn export_jsonl(store: &LocalFirstStore) -> Result<String, LocalFirstError> {
    store.queue().export_logical_v1()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn export_import_reconstructs_remote_state() {
        let source_dir = tempdir().unwrap();
        let target_dir = tempdir().unwrap();
        let mut source = LocalFirstStore::open(source_dir.path(), "source").unwrap();
        let mut target = LocalFirstStore::open(target_dir.path(), "target").unwrap();

        source.set("doc:1", serde_json::json!({"v": 1})).unwrap();
        source.set("doc:2", serde_json::json!({"v": 2})).unwrap();

        let export = export_jsonl(&source).unwrap();
        assert_eq!(import_jsonl(&mut target, &export).unwrap(), 2);
        assert_eq!(target.get("doc:1"), source.get("doc:1"));
        assert_eq!(target.get("doc:2"), source.get("doc:2"));
        assert_eq!(target.queue().logical_changes(), source.queue().logical_changes());
    }

    #[test]
    fn reimport_is_idempotent() {
        let source_dir = tempdir().unwrap();
        let target_dir = tempdir().unwrap();
        let mut source = LocalFirstStore::open(source_dir.path(), "source").unwrap();
        let mut target = LocalFirstStore::open(target_dir.path(), "target").unwrap();
        source.set("doc:1", serde_json::json!(1)).unwrap();
        let export = export_jsonl(&source).unwrap();
        assert_eq!(import_jsonl(&mut target, &export).unwrap(), 1);
        assert_eq!(import_jsonl(&mut target, &export).unwrap(), 0);
    }

    #[test]
    fn malformed_jsonl_reports_line_number() {
        let dir = tempdir().unwrap();
        let mut store = LocalFirstStore::open(dir.path(), "target").unwrap();
        let error = import_jsonl(&mut store, "{}\nnot-json\n").unwrap_err();
        assert!(error.to_string().contains("line 2"));
    }
}
