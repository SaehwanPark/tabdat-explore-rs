use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{
  CellValue, ExecutionResult, JoinResult, PreviewResult, RuntimeError, Session,
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after the Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-join-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("patients.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (30, 22.5, 'F', 100.0), (42, 25.0, 'M', 150.0), (54, 27.5, 'F', NULL)) AS patients(age, bmi, sex, cost)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    Self { root, parquet }
  }

  fn use_cmd(&self) -> String {
    format!("use {}", self.parquet.to_string_lossy())
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

#[test]
fn join_requires_an_active_dataset() {
  let mut session = Session::new();
  let command = parse_command("join lookup on id").expect("valid command");
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "join" }
  );
}

#[test]
fn join_reports_unknown_named_table() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let command = parse_command("join lookup on sex").expect("valid command");
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnknownTable {
      name: "lookup".to_string()
    }
  );
}

#[test]
fn join_reports_unknown_variable_errors() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(
      parse_command("sql select sex, count(*) as n from active group by sex into lookup").unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  // Missing from active dataset
  let command = parse_command("join lookup on missing").expect("valid command");
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::JoinUnknownVariable {
      variables: vec!["missing".to_string()]
    }
  );

  // Missing from named table
  let command = parse_command("join lookup on age").expect("valid command");
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::JoinUnknownVariableInTable {
      table_name: "lookup".to_string(),
      variables: vec!["age".to_string()]
    }
  );
}

#[test]
fn inner_join_named_table() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(
      parse_command(
        "sql select sex, avg(bmi) as mean_bmi from active group by sex into sex_lookup",
      )
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("join sex_lookup on sex").unwrap())
    .unwrap();

  let ExecutionResult::Join(JoinResult { dataset }) = result else {
    panic!("expected Join result, got {result:?}");
  };
  assert_eq!(dataset.row_count, 3);
  let column_names: Vec<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(column_names, vec!["age", "bmi", "sex", "cost", "mean_bmi"]);

  let preview = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult { rows, .. }) = preview else {
    panic!("expected Head result");
  };
  assert_eq!(
    rows,
    vec![
      vec![
        CellValue::SignedInteger(30),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 225,
        },
        CellValue::Text("F".to_string()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1000,
        },
        CellValue::Float(25.0)
      ],
      vec![
        CellValue::SignedInteger(42),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 250,
        },
        CellValue::Text("M".to_string()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1500,
        },
        CellValue::Float(25.0)
      ],
      vec![
        CellValue::SignedInteger(54),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 275,
        },
        CellValue::Text("F".to_string()),
        CellValue::Null,
        CellValue::Float(25.0)
      ],
    ]
  );
}

#[test]
fn left_join_preserves_active_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(
      parse_command(
        "sql select 'F' as sex, 'matched' as label from active limit 1 into female_lookup",
      )
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("join female_lookup on sex, how=left").unwrap())
    .unwrap();

  let ExecutionResult::Join(JoinResult { dataset }) = result else {
    panic!("expected Join result, got {result:?}");
  };
  assert_eq!(dataset.row_count, 3);

  let preview = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult { rows, .. }) = preview else {
    panic!("expected Head result");
  };
  assert_eq!(
    rows,
    vec![
      vec![
        CellValue::SignedInteger(30),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 225,
        },
        CellValue::Text("F".to_string()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1000,
        },
        CellValue::Text("matched".to_string())
      ],
      vec![
        CellValue::SignedInteger(42),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 250,
        },
        CellValue::Text("M".to_string()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1500,
        },
        CellValue::Null
      ],
      vec![
        CellValue::SignedInteger(54),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 275,
        },
        CellValue::Text("F".to_string()),
        CellValue::Null,
        CellValue::Text("matched".to_string())
      ],
    ]
  );
}

#[test]
fn join_supports_multiple_keys_and_collision_suffix() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(
      parse_command("sql select sex, age, cost, age + 1 as next_age from active into lookup")
        .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("join lookup on sex age, suffix(_lookup)").unwrap())
    .unwrap();

  let ExecutionResult::Join(JoinResult { dataset }) = result else {
    panic!("expected Join result");
  };
  assert_eq!(dataset.row_count, 3);
  let column_names: Vec<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(
    column_names,
    vec!["age", "bmi", "sex", "cost", "cost_lookup", "next_age"]
  );
}

#[test]
fn join_suffixing_keeps_output_names_unique() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select sex, cost from active into lookup").unwrap())
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("generate cost_lookup = 1").unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("join lookup on sex, suffix(_lookup)").unwrap())
    .unwrap();

  let ExecutionResult::Join(JoinResult { dataset }) = result else {
    panic!("expected Join result");
  };
  let column_names: Vec<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(
    column_names,
    vec!["age", "bmi", "sex", "cost", "cost_lookup", "cost_lookup_2"]
  );
}

#[test]
fn join_preserves_active_and_match_sequence() {
  let fixture = Fixture::new();
  let path = fixture.root.join("join_order.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT join_key, left_label FROM (VALUES (1, 'A', 'left-a1'), (2, 'B', 'left-b'), (3, 'A', 'left-a2'), (4, 'C', 'left-c')) AS active_rows(row_id, join_key, left_label) ORDER BY row_id) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  // Test Inner Join
  {
    let mut session = Session::new();
    session
      .execute(parse_command(&format!("use {path_str}")).unwrap())
      .unwrap();
    session
      .execute(
        parse_command(
          "sql select join_key, right_label FROM (VALUES (1, 'A', 'a-first'), (2, 'A', 'a-second'), (3, 'B', 'b-only')) AS lookup_rows(row_id, join_key, right_label) ORDER BY row_id into lookup",
        )
        .unwrap(),
      )
      .unwrap();
    session
      .execute(parse_command(&format!("use {path_str}")).unwrap())
      .unwrap();

    let result = session
      .execute(parse_command("join lookup on join_key").unwrap())
      .unwrap();
    let ExecutionResult::Join(JoinResult { dataset }) = result else {
      panic!("expected Join result");
    };
    assert_eq!(dataset.row_count, 5);

    let preview = session.execute(parse_command("head 10").unwrap()).unwrap();
    let ExecutionResult::Head(PreviewResult { rows, .. }) = preview else {
      panic!("expected Head result");
    };
    assert_eq!(
      rows,
      vec![
        vec![
          CellValue::Text("A".to_string()),
          CellValue::Text("left-a1".to_string()),
          CellValue::Text("a-first".to_string()),
        ],
        vec![
          CellValue::Text("A".to_string()),
          CellValue::Text("left-a1".to_string()),
          CellValue::Text("a-second".to_string()),
        ],
        vec![
          CellValue::Text("B".to_string()),
          CellValue::Text("left-b".to_string()),
          CellValue::Text("b-only".to_string()),
        ],
        vec![
          CellValue::Text("A".to_string()),
          CellValue::Text("left-a2".to_string()),
          CellValue::Text("a-first".to_string()),
        ],
        vec![
          CellValue::Text("A".to_string()),
          CellValue::Text("left-a2".to_string()),
          CellValue::Text("a-second".to_string()),
        ],
      ]
    );
  }

  // Test Left Join
  {
    let mut session = Session::new();
    session
      .execute(parse_command(&format!("use {path_str}")).unwrap())
      .unwrap();
    session
      .execute(
        parse_command(
          "sql select join_key, right_label FROM (VALUES (1, 'A', 'a-first'), (2, 'A', 'a-second'), (3, 'B', 'b-only')) AS lookup_rows(row_id, join_key, right_label) ORDER BY row_id into lookup",
        )
        .unwrap(),
      )
      .unwrap();
    session
      .execute(parse_command(&format!("use {path_str}")).unwrap())
      .unwrap();

    let result = session
      .execute(parse_command("join lookup on join_key, how=left").unwrap())
      .unwrap();
    let ExecutionResult::Join(JoinResult { dataset }) = result else {
      panic!("expected Join result");
    };
    assert_eq!(dataset.row_count, 6);

    let preview = session.execute(parse_command("head 10").unwrap()).unwrap();
    let ExecutionResult::Head(PreviewResult { rows, .. }) = preview else {
      panic!("expected Head result");
    };
    assert_eq!(
      rows,
      vec![
        vec![
          CellValue::Text("A".to_string()),
          CellValue::Text("left-a1".to_string()),
          CellValue::Text("a-first".to_string()),
        ],
        vec![
          CellValue::Text("A".to_string()),
          CellValue::Text("left-a1".to_string()),
          CellValue::Text("a-second".to_string()),
        ],
        vec![
          CellValue::Text("B".to_string()),
          CellValue::Text("left-b".to_string()),
          CellValue::Text("b-only".to_string()),
        ],
        vec![
          CellValue::Text("A".to_string()),
          CellValue::Text("left-a2".to_string()),
          CellValue::Text("a-first".to_string()),
        ],
        vec![
          CellValue::Text("A".to_string()),
          CellValue::Text("left-a2".to_string()),
          CellValue::Text("a-second".to_string()),
        ],
        vec![
          CellValue::Text("C".to_string()),
          CellValue::Text("left-c".to_string()),
          CellValue::Null,
        ],
      ]
    );
  }
}

#[test]
fn join_uses_collision_free_internal_order_columns() {
  let fixture = Fixture::new();
  let path = fixture.root.join("join_order_collision.parquet");
  let path_str = path.to_string_lossy().into_owned();
  let conn = Connection::open_in_memory().unwrap();
  conn
    .execute(
      "COPY (SELECT join_key, __tabdat_join_order, left_label FROM (VALUES (1, 'A', 101, 'left-a'), (2, 'B', 102, 'left-b')) AS active_rows(row_id, join_key, __tabdat_join_order, left_label) ORDER BY row_id) TO ? (FORMAT PARQUET)",
      [&path_str],
    )
    .unwrap();

  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();
  session
    .execute(
      parse_command(
        "sql select join_key, __tabdat_join_right_order, right_label FROM (VALUES (1, 'A', 201, 'right-a'), (2, 'B', 202, 'right-b')) AS lookup_rows(row_id, join_key, __tabdat_join_right_order, right_label) ORDER BY row_id into lookup",
      )
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&format!("use {path_str}")).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("join lookup on join_key").unwrap())
    .unwrap();
  let ExecutionResult::Join(JoinResult { dataset }) = result else {
    panic!("expected Join result");
  };
  let column_names: Vec<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(
    column_names,
    vec![
      "join_key",
      "__tabdat_join_order",
      "left_label",
      "__tabdat_join_right_order",
      "right_label"
    ]
  );

  let preview = session.execute(parse_command("head 5").unwrap()).unwrap();
  let ExecutionResult::Head(PreviewResult { rows, .. }) = preview else {
    panic!("expected Head result");
  };
  assert_eq!(
    rows,
    vec![
      vec![
        CellValue::Text("A".to_string()),
        CellValue::SignedInteger(101),
        CellValue::Text("left-a".to_string()),
        CellValue::SignedInteger(201),
        CellValue::Text("right-a".to_string()),
      ],
      vec![
        CellValue::Text("B".to_string()),
        CellValue::SignedInteger(102),
        CellValue::Text("left-b".to_string()),
        CellValue::SignedInteger(202),
        CellValue::Text("right-b".to_string()),
      ],
    ]
  );
}

#[test]
fn failed_join_validation_preserves_active_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select sex from active into missing_label").unwrap())
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();

  let before = session.active_dataset().unwrap().clone();

  // 1. Unknown table
  assert!(
    session
      .execute(parse_command("join missing_table on sex").unwrap())
      .is_err()
  );
  assert_eq!(session.active_dataset().unwrap(), &before);

  // 2. Unknown key in active dataset
  assert!(
    session
      .execute(parse_command("join missing_label on missing").unwrap())
      .is_err()
  );
  assert_eq!(session.active_dataset().unwrap(), &before);

  // 3. Unknown key in named table
  assert!(
    session
      .execute(parse_command("join missing_label on age").unwrap())
      .is_err()
  );
  assert_eq!(session.active_dataset().unwrap(), &before);
}

#[test]
fn join_retains_left_variable_labels() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable age \"Patient Age\"").unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable sex \"Biological Sex\"").unwrap())
    .unwrap();
  session
    .execute(
      parse_command(
        "sql select sex, avg(bmi) as mean_bmi from active group by sex into sex_lookup",
      )
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable age \"Patient Age\"").unwrap())
    .unwrap();
  session
    .execute(parse_command("label variable sex \"Biological Sex\"").unwrap())
    .unwrap();

  session
    .execute(parse_command("join sex_lookup on sex").unwrap())
    .unwrap();

  let labels = session
    .active_label_metadata()
    .expect("labels should exist");
  assert_eq!(
    labels.variable_labels,
    vec![
      ("age".to_string(), "Patient Age".to_string()),
      ("sex".to_string(), "Biological Sex".to_string()),
    ]
  );
}

#[test]
fn join_syncs_active_named_table() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(
      parse_command(
        "sql select sex, avg(bmi) as mean_bmi from active group by sex into sex_lookup",
      )
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command(&fixture.use_cmd()).unwrap())
    .unwrap();
  session
    .execute(parse_command("sql select * from active into my_active").unwrap())
    .unwrap();

  assert_eq!(session.active_table_name(), Some("my_active"));
  assert_eq!(
    session
      .named_tables()
      .get("my_active")
      .unwrap()
      .columns
      .len(),
    4
  );

  session
    .execute(parse_command("join sex_lookup on sex").unwrap())
    .unwrap();

  assert_eq!(session.active_table_name(), Some("my_active"));
  let updated = session.named_tables().get("my_active").unwrap();
  assert_eq!(updated.columns.len(), 5);
  assert_eq!(updated.row_count, 3);
}

#[test]
fn join_executes_in_multiline_script() {
  let fixture = Fixture::new();
  let script_path = fixture.root.join("analysis.td");
  let script_content = format!(
    "use {}\n\
     sql \"\"\"\n\
     select sex, avg(bmi) as mean_bmi\n\
     from active\n\
     group by sex\n\
     \"\"\" into sex_lookup\n\
     use {}\n\
     join sex_lookup on sex\n",
    fixture.parquet.to_string_lossy(),
    fixture.parquet.to_string_lossy(),
  );
  fs::write(&script_path, script_content).expect("script should be written");

  let mut session = Session::new();
  let result = session
    .execute_run(&script_path)
    .expect("script should run");
  let ExecutionResult::Run(run_result) = result else {
    panic!("expected Run result");
  };
  assert_eq!(run_result.executed_commands, 4);

  let active = session
    .active_dataset()
    .expect("active dataset should exist");
  assert_eq!(active.row_count, 3);
  let column_names: Vec<&str> = active.columns.iter().map(|c| c.name.as_str()).collect();
  assert_eq!(column_names, vec!["age", "bmi", "sex", "cost", "mean_bmi"]);
}
