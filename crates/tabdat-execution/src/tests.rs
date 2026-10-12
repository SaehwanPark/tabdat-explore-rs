use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_MISSING_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn describe_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session.execute(Command::Describe).unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "describe"
    }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn datasignature_float_hex_matches_python_for_edge_values() {
  assert_eq!(python_float_hex(0.0), "0x0.0p+0");
  assert_eq!(python_float_hex(-0.0), "-0x0.0p+0");
  assert_eq!(python_float_hex(1.0), "0x1.0000000000000p+0");
  assert_eq!(
    python_float_hex(f64::from_bits(1)),
    "0x0.0000000000001p-1022"
  );
  assert_eq!(python_float_hex(f64::NAN), "nan");
  assert_eq!(python_float_hex(f64::INFINITY), "inf");
  assert_eq!(python_float_hex(f64::NEG_INFINITY), "-inf");
}

#[test]
fn count_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session.execute(Command::Count).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "count" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn datasignature_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session.execute(Command::Datasignature).unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "datasignature"
    }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn assert_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Assert {
        expression: AssertExpression::Binary {
          left: Box::new(AssertExpression::Identifier("age".to_owned())),
          operator: AssertBinaryOperator::Greater,
          right: Box::new(AssertExpression::Number("0".to_owned())),
        },
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "assert" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn keep_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Keep {
        variables: vec!["age".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "keep" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn select_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Select {
        variables: vec!["age".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "select" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn head_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Head {
        limit: RowLimit::default(),
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "head" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn tail_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Tail {
        limit: RowLimit::default(),
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "tail" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn codebook_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Codebook {
        variables: Vec::new()
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "codebook"
    }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn missing_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Missing {
        variables: Vec::new(),
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "missing" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn duplicates_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Duplicates {
        variables: Vec::new(),
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "duplicates"
    }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn isid_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Isid {
        variables: vec!["id".to_owned()],
        missok: false,
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "isid" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn replace_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Replace {
        variable: "age".to_owned(),
        expression: GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("age".to_owned())),
          operator: GenerateBinaryOperator::Add,
          right: Box::new(GenerateExpression::Number("1".to_owned())),
        },
        condition: None,
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "replace" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn rename_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Rename {
        old_name: "old_name".to_owned(),
        new_name: "new_name".to_owned(),
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "rename" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn sort_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Sort {
        variables: vec!["value".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "sort" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn gsort_does_not_initialize_backend_for_a_new_session() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(Command::Gsort {
        keys: vec![SortKey {
          variable: "value".to_owned(),
          descending: true,
        }],
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "gsort" }
  );
  assert!(session.backend.is_none());
  assert!(session.active_dataset.is_none());
}

#[test]
fn failed_preview_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
    ))
    .expect("the test active relation should be created");
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
    .expect("the test active relation should be dropped");

  assert_eq!(
    session
      .execute(Command::Head {
        limit: RowLimit::default(),
      })
      .unwrap_err(),
    RuntimeError::PreviewFailed { command: "head" }
  );
  assert_eq!(
    session
      .execute(Command::Tail {
        limit: RowLimit::default(),
      })
      .unwrap_err(),
    RuntimeError::PreviewFailed { command: "tail" }
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
}

#[test]
fn failed_keep_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "age".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());

  assert_eq!(
    session
      .execute(Command::Keep {
        variables: vec!["age".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::KeepFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  assert!(session.backend.is_some());
}

#[test]
fn failed_drop_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![
      ColumnInfo {
        name: "age".to_owned(),
        data_type: "INTEGER".to_owned(),
      },
      ColumnInfo {
        name: "sex".to_owned(),
        data_type: "VARCHAR".to_owned(),
      },
    ],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS age"
    ))
    .expect("the mismatched active relation should be created");

  assert_eq!(
    session
      .execute(Command::Drop {
        variables: vec!["age".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::DropFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  let age: i64 = session
    .backend
    .as_ref()
    .expect("test backend should exist")
    .query_i64(&format!("SELECT age FROM {ACTIVE_TABLE}"))
    .expect("the pre-existing active relation should remain available");
  assert_eq!(age, 7);
}

#[test]
fn failed_select_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![
      ColumnInfo {
        name: "age".to_owned(),
        data_type: "INTEGER".to_owned(),
      },
      ColumnInfo {
        name: "sex".to_owned(),
        data_type: "VARCHAR".to_owned(),
      },
    ],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS age"
    ))
    .expect("the mismatched active relation should be created");

  assert_eq!(
    session
      .execute(Command::Select {
        variables: vec!["sex".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::SelectFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  let age: i64 = session
    .backend
    .as_ref()
    .expect("test backend should exist")
    .query_i64(&format!("SELECT age FROM {ACTIVE_TABLE}"))
    .expect("the pre-existing active relation should remain available");
  assert_eq!(age, 7);
}

#[test]
fn failed_summary_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());

  assert_eq!(
    session
      .execute(Command::Summarize {
        variables: vec!["value".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::SummaryFailed {
      variable: "value".to_owned(),
    }
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
}

#[test]
fn failed_codebook_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
    ))
    .expect("the test active relation should be created");
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
    .expect("the test active relation should be dropped");

  assert_eq!(
    session
      .execute(Command::Codebook {
        variables: vec!["value".to_owned()]
      })
      .unwrap_err(),
    RuntimeError::CodebookFailed {
      variable: "value".to_owned()
    }
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
}

#[test]
fn failed_missing_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
    ))
    .expect("the test active relation should be created");
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
    .expect("the test active relation should be dropped");

  assert_eq!(
    session
      .execute(Command::Missing {
        variables: vec!["value".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::MissingFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
}

#[test]
fn failed_duplicates_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
    ))
    .expect("the test active relation should be created");
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
    .expect("the test active relation should be dropped");

  assert_eq!(
    session
      .execute(Command::Duplicates {
        variables: vec!["value".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::DuplicatesFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
}

#[test]
fn failed_isid_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
    ))
    .expect("the test active relation should be created");
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
    .expect("the test active relation should be dropped");

  assert_eq!(
    session
      .execute(Command::Isid {
        variables: vec!["value".to_owned()],
        missok: false,
      })
      .unwrap_err(),
    RuntimeError::IsidFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
}

#[test]
fn failed_datasignature_keeps_the_published_dataset_metadata() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
    ))
    .expect("the test active relation should be created");
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
    .expect("the test active relation should be dropped");

  assert_eq!(
    session.execute(Command::Datasignature).unwrap_err(),
    RuntimeError::DatasignatureFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
}

#[test]
fn failed_generate_keeps_metadata_and_private_active_relation() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
    ))
    .expect("the mismatched active relation should be created");

  assert_eq!(
    session
      .execute(Command::Generate {
        variable: "value2".to_owned(),
        expression: GenerateExpression::Identifier("value".to_owned()),
      })
      .unwrap_err(),
    RuntimeError::GenerateFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  let value: i64 = session
    .backend
    .as_ref()
    .expect("test backend should exist")
    .query_i64(&format!("SELECT other FROM {ACTIVE_TABLE}"))
    .expect("the prior active relation should remain queryable");
  assert_eq!(value, 7);
}

#[test]
fn failed_replace_keeps_metadata_and_private_active_relation() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
    ))
    .expect("the mismatched active relation should be created");

  assert_eq!(
    session
      .execute(Command::Replace {
        variable: "value".to_owned(),
        expression: GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("value".to_owned())),
          operator: GenerateBinaryOperator::Add,
          right: Box::new(GenerateExpression::Number("1".to_owned())),
        },
        condition: None,
      })
      .unwrap_err(),
    RuntimeError::ReplaceFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  let value: i64 = session
    .backend
    .as_ref()
    .expect("test backend should exist")
    .query_i64(&format!("SELECT other FROM {ACTIVE_TABLE}"))
    .expect("the prior active relation should remain queryable");
  assert_eq!(value, 7);
}

#[test]
fn failed_rename_keeps_metadata_and_private_active_relation() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
    ))
    .expect("the mismatched active relation should be created");

  assert_eq!(
    session
      .execute(Command::Rename {
        old_name: "value".to_owned(),
        new_name: "renamed".to_owned(),
      })
      .unwrap_err(),
    RuntimeError::RenameFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  let value: i64 = session
    .backend
    .as_ref()
    .expect("test backend should exist")
    .query_i64(&format!("SELECT other FROM {ACTIVE_TABLE}"))
    .expect("the prior active relation should remain queryable");
  assert_eq!(value, 7);
}

#[test]
fn failed_sort_keeps_metadata_and_private_active_relation() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
    ))
    .expect("the mismatched active relation should be created");

  assert_eq!(
    session
      .execute(Command::Sort {
        variables: vec!["value".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::SortFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  let value: i64 = session
    .backend
    .as_ref()
    .expect("test backend should exist")
    .query_i64(&format!("SELECT other FROM {ACTIVE_TABLE}"))
    .expect("the prior active relation should remain queryable");
  assert_eq!(value, 7);
}

#[test]
fn failed_gsort_keeps_metadata_and_private_active_relation() {
  let mut session = Session::new();
  session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
  let dataset = DatasetInfo {
    source: PathBuf::from("fixture.parquet"),
    row_count: 1,
    columns: vec![ColumnInfo {
      name: "value".to_owned(),
      data_type: "INTEGER".to_owned(),
    }],
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
  };
  session.active_dataset = Some(dataset.clone());
  session
    .backend
    .as_mut()
    .expect("test backend should exist")
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
    ))
    .expect("the mismatched active relation should be created");

  assert_eq!(
    session
      .execute(Command::Gsort {
        keys: vec![SortKey {
          variable: "value".to_owned(),
          descending: true,
        }],
      })
      .unwrap_err(),
    RuntimeError::GsortFailed
  );
  assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  let value: i64 = session
    .backend
    .as_ref()
    .expect("test backend should exist")
    .query_i64(&format!("SELECT other FROM {ACTIVE_TABLE}"))
    .expect("the prior active relation should remain queryable");
  assert_eq!(value, 7);
}

#[test]
fn failed_staged_read_keeps_the_private_active_relation() {
  let mut backend = DuckDbBackend::new().expect("test backend should initialize");
  backend
    .execute_batch(&format!(
      "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
    ))
    .expect("the test active relation should be created");

  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock should be after the Unix epoch")
    .as_nanos();
  let fixture_id = NEXT_MISSING_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
  let path = std::env::temp_dir().join(format!(
    "tabdat-runtime-missing-{}-{nonce}-{fixture_id}.parquet",
    std::process::id()
  ));
  assert!(!path.exists());
  assert!(matches!(
    backend.load_eager_parquet(&path),
    Err(RuntimeError::ParquetRead { .. })
  ));

  let value: i64 = backend
    .query_i64(&format!("SELECT value FROM {ACTIVE_TABLE}"))
    .expect("the prior active relation should remain queryable");
  assert_eq!(value, 7);
}
