mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn example(name: &str) -> PathBuf {
  root().join("examples/mma21p1panel").join(name)
}
fn prepare(d: &TestDir) {
  fs::write(
    d.path().join("source.dat"),
    include_str!("fixtures/mma21_between_figure/source.dat"),
  )
  .unwrap();
  let o = Command::new("sh")
    .current_dir(d.path())
    .arg(example("prepare.sh"))
    .args(["source.dat", "MOM.csv"])
    .output()
    .unwrap();
  ok(&o);
  fs::create_dir(d.path().join("reference")).unwrap();
  for (name, content) in [
    (
      "source.csv",
      include_str!("fixtures/mma21_between_figure/source.csv"),
    ),
    (
      "rows.csv",
      include_str!("fixtures/mma21_between_figure/rows.csv"),
    ),
    (
      "meta.csv",
      include_str!("fixtures/mma21_between_figure/meta.csv"),
    ),
  ] {
    fs::write(d.path().join("reference").join(name), content).unwrap();
  }
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(example("between_figure.td"))
    .output()
    .unwrap()
}
fn audit(d: &TestDir) -> Output {
  Command::new("python3")
    .arg(example("check_between_figure.py"))
    .arg(d.path())
    .arg(d.path().join("reference"))
    .arg(d.path().join("out.jsonl"))
    .output()
    .unwrap()
}
fn svg_audit(d: &TestDir) -> Output {
  Command::new("python3").args(["-c","import sys;sys.path.insert(0,sys.argv[1]);from check_between_figure import check_svg,rows;check_svg(sys.argv[2],rows(sys.argv[3]))"]).arg(example("")).arg(d.path().join("between_figure.svg")).arg(d.path().join("between_figure.csv")).output().unwrap()
}
fn ok(o: &Output) {
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}
fn events(o: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&o.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn clear(d: &TestDir) {
  for f in ["between_figure.csv", "between_figure.svg"] {
    fs::remove_file(d.path().join(f)).unwrap();
  }
}
fn rewrite(input: &str, mut f: impl FnMut(usize, &mut Vec<String>)) -> String {
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let records = lines
    .enumerate()
    .map(|(i, l)| {
      let mut fields = l.split(',').map(str::to_owned).collect();
      f(i, &mut fields);
      fields.join(",")
    })
    .collect::<Vec<_>>();
  format!("{header}\n{}\n", records.join("\n"))
}

#[test]
fn independent_means_rank_fits_svg_modes_order_and_no_clobber() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("MOM.csv");
  let input = fs::read_to_string(&path).unwrap();
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let csv = fs::read(d.path().join("between_figure.csv")).unwrap();
  let svg = fs::read(d.path().join("between_figure.svg")).unwrap();
  assert_eq!(fs::read_to_string(&path).unwrap(), input);
  let o = run(&d, "--json");
  assert!(!o.status.success());
  assert!(String::from_utf8_lossy(&o.stderr).contains("export target already exists"));
  assert_eq!(fs::read(d.path().join("between_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("between_figure.svg")).unwrap(), svg);
  clear(&d);
  ok(&run(&d, "--text"));
  assert_eq!(fs::read(d.path().join("between_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("between_figure.svg")).unwrap(), svg);
  clear(&d);
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let mut records: Vec<_> = lines.collect();
  records.reverse();
  fs::write(&path, format!("{header}\n{}\n", records.join("\n"))).unwrap();
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  assert_eq!(fs::read(d.path().join("between_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("between_figure.svg")).unwrap(), svg);
  clear(&d);
  fs::write(d.path().join("between_figure.svg"), b"sentinel").unwrap();
  let o = run(&d, "--json");
  assert!(!o.status.success());
  assert_eq!(
    fs::read(d.path().join("between_figure.svg")).unwrap(),
    b"sentinel"
  );
  assert_eq!(fs::read(d.path().join("between_figure.csv")).unwrap(), csv);
}

#[test]
fn joint_finite_rows_and_nonmodel_missingness() {
  let d = TestDir::new();
  prepare(&d);
  let p = d.path().join("MOM.csv");
  let input = fs::read_to_string(&p).unwrap();
  let changed = rewrite(&input, |i, r| match i {
    0 => r[1] = "".into(),
    5 => r[2] = "".into(),
    8 => r[3] = "".into(),
    92.. => r[2] = "".into(),
    _ => {}
  });
  fs::write(&p, &changed).unwrap();
  for (name, content) in [
    (
      "source.csv",
      include_str!("fixtures/mma21_between_figure_missing/source.csv"),
    ),
    (
      "rows.csv",
      include_str!("fixtures/mma21_between_figure_missing/rows.csv"),
    ),
    (
      "meta.csv",
      include_str!("fixtures/mma21_between_figure_missing/meta.csv"),
    ),
  ] {
    fs::write(d.path().join("reference").join(name), content).unwrap();
  }
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let data = fs::read_to_string(d.path().join("between_figure.csv")).unwrap();
  let rows: Vec<Vec<&str>> = data
    .lines()
    .skip(1)
    .map(|s| s.split(',').collect())
    .collect();
  assert_eq!(rows.len(), 23);
  assert_eq!(rows[0][1], "3");
  assert_eq!(rows[0][2].parse::<f64>().unwrap(), 1980.0);
  assert_eq!(rows[1][1], "3");
  assert_eq!(rows[2][1], "4");
  let source: Vec<Vec<&str>> = changed
    .lines()
    .skip(1)
    .map(|s| s.split(',').collect())
    .collect();
  let count = source[1..4].len() as f64;
  let cents: i64 = source[1..4]
    .iter()
    .map(|r| (r[2].parse::<f64>().unwrap() * 100.0).round() as i64)
    .sum();
  assert_eq!(
    rows[0][5].parse::<f64>().unwrap(),
    (cents as f64 / 100.0) / count
  );
  assert_eq!(fs::read_to_string(p).unwrap(), changed);
}

#[test]
fn invalid_precision_keys_collision_and_singular_design_do_not_export() {
  let d = TestDir::new();
  prepare(&d);
  let p = d.path().join("MOM.csv");
  let input = fs::read_to_string(&p).unwrap();
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let rows: Vec<_> = lines.collect();
  let svg = d.path().join("between_figure.svg");
  fs::write(&svg, b"sentinel").unwrap();
  let cases = [
    format!("{header}\n{}\n{}\n", rows.join("\n"), rows[0]),
    rewrite(&input, |i, r| {
      if i == 0 {
        r[7] = "1.5".into()
      }
    }),
    rewrite(&input, |i, r| {
      if i == 0 {
        r[2] = "1.001".into()
      }
    }),
    rewrite(&input, |i, r| {
      if i == 0 {
        r[2] = "1000000.01".into()
      }
    }),
    format!(
      "{header},FIGURE_LOWESS\n{}\n",
      rows
        .iter()
        .map(|r| format!("{r},999"))
        .collect::<Vec<_>>()
        .join("\n")
    ),
    rewrite(&input, |_, r| r[2] = "1.00".into()),
  ];
  for case in cases {
    fs::write(&p, &case).unwrap();
    let o = run(&d, "--json");
    assert!(!o.status.success());
    assert!(
      !events(&o)
        .iter()
        .any(|e| e["result_type"] == "RegressionResult")
    );
    assert_eq!(fs::read_to_string(&p).unwrap(), case);
    assert!(!d.path().join("between_figure.csv").exists());
    assert_eq!(fs::read(&svg).unwrap(), b"sentinel");
  }
}

#[test]
fn checker_rejects_floating_average_shared_wrong_values_and_svg_forgeries() {
  let d = TestDir::new();
  prepare(&d);
  let wrong = fs::read_to_string(example("between_figure.td"))
    .unwrap()
    .replace(
      "SUM(CAST(lnhr AS DECIMAL(18,2)))/COUNT(*)",
      "AVG(lnhr ORDER BY year)",
    )
    .replace(
      "SUM(CAST(lnwg AS DECIMAL(18,2)))/COUNT(*)",
      "AVG(lnwg ORDER BY year)",
    );
  fs::write(d.path().join("wrong.td"), wrong).unwrap();
  let o = tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-f", "wrong.td"])
    .output()
    .unwrap();
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  assert!(!audit(&d).status.success());
  clear(&d);
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let p = d.path().join("between_figure.svg");
  let svg = fs::read_to_string(&p).unwrap();
  let start = svg.find("class=\"line-series\"").unwrap();
  let pos = start + svg[start..].find(" d=\"").unwrap() + 4;
  let first_l = pos + svg[pos..].find('L').unwrap();
  let mut bad = svg.clone();
  bad.replace_range(first_l..first_l + 1, "M");
  fs::write(&p, &bad).unwrap();
  assert!(!svg_audit(&d).status.success());
  fs::write(&p, &svg).unwrap();
  let start = svg.find("class=\"scatter-point\"").unwrap();
  let pos = start + svg[start..].find(" cx=\"").unwrap() + 5;
  let end = pos + svg[pos..].find('"').unwrap();
  let mut bad = svg.clone();
  bad.replace_range(pos..end, "0");
  fs::write(&p, bad).unwrap();
  assert!(!svg_audit(&d).status.success());
  fs::write(&p, svg).unwrap();
  for f in ["between_figure.csv", "reference/rows.csv"] {
    let p = d.path().join(f);
    let text = fs::read_to_string(&p).unwrap();
    let mut lines = text.lines();
    let header = lines.next().unwrap();
    let data = lines
      .enumerate()
      .map(|(i, l)| {
        if i == 0 {
          let mut v: Vec<_> = l.split(',').collect();
          v[7] = "999";
          v.join(",")
        } else {
          l.to_owned()
        }
      })
      .collect::<Vec<_>>()
      .join("\n");
    fs::write(p, format!("{header}\n{data}\n")).unwrap();
  }
  let o = audit(&d);
  assert!(!o.status.success());
  assert!(String::from_utf8_lossy(&o.stderr).contains("equation"));
}

#[test]
fn smoother_export_labels_and_plot_preserve_between_ols_state() {
  fn exec(s: &mut Session, command: &str) {
    s.execute(parse_command(command).unwrap()).unwrap();
  }
  let d = TestDir::new();
  prepare(&d);
  let mut s = Session::new();
  exec(
    &mut s,
    &format!("use \"{}\"", d.path().join("MOM.csv").display()),
  );
  exec(
    &mut s,
    "sql \"\"\"SELECT id,COUNT(*) AS model_rows,MIN(year) AS first_year,MAX(year) AS last_year,SUM(CAST(lnhr AS DECIMAL(18,2)))/COUNT(*) AS mean_lnhr,SUM(CAST(lnwg AS DECIMAL(18,2)))/COUNT(*) AS mean_lnwg FROM active WHERE isfinite(lnhr) AND isfinite(lnwg) GROUP BY id ORDER BY id\"\"\" into between_figure_data",
  );
  exec(&mut s, "regress mean_lnhr mean_lnwg");
  let fit = s.state().estimation.clone();
  exec(&mut s, "predict figure_linear, xb");
  exec(
    &mut s,
    "lowess mean_lnhr mean_lnwg, gen(figure_lowess) bandwidth=0.8",
  );
  exec(&mut s, "summarize figure_linear figure_lowess");
  exec(
    &mut s,
    &format!("export \"{}\"", d.path().join("state.csv").display()),
  );
  exec(&mut s, "label variable mean_lnwg \"Mean log hourly wage\"");
  exec(&mut s, "label variable mean_lnhr \"Mean log annual hours\"");
  exec(&mut s, "label variable figure_lowess \"Nonparametric fit\"");
  exec(
    &mut s,
    "label variable figure_linear \"Between linear fit\"",
  );
  exec(
    &mut s,
    &format!(
      "scatter mean_lnhr mean_lnwg, lines(figure_lowess figure_linear) saving(\"{}\") noopen",
      d.path().join("state.svg").display()
    ),
  );
  assert_eq!(s.state().estimation, fit);
  exec(&mut s, "predict after_lowess, xb");
  assert_eq!(s.state().estimation, fit);
}
