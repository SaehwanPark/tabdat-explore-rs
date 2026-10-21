mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs};

#[test]
fn current_row_pmf_cdf_offset_exports_match_r_and_covariance_does_not_change_the_law() {
  let dir = TestDir::new();
  fs::write(
    dir.path().join("data.csv"),
    include_str!("../examples/nb2-offset/nb2_offset.csv"),
  )
  .unwrap();
  let mut compared = 0;
  for (command, family) in [("poisson", "poisson"), ("nbreg", "nb2")] {
    let refs: BTreeMap<_, (f64, f64)> =
      include_str!("../crates/tabdat-stats/tests/fixtures/count_point_workflow_reference.csv")
        .lines()
        .skip(1)
        .map(|s| s.split(',').collect::<Vec<_>>())
        .filter(|r| r[0] == family)
        .map(|r| {
          (
            (
              r[1].parse::<usize>().unwrap(),
              r[2].parse::<usize>().unwrap(),
            ),
            (r[3].parse().unwrap(), r[4].parse().unwrap()),
          )
        })
        .collect();
    let mut baseline = None;
    for covariance in ["", " robust", " cluster(person)"] {
      let script = format!(
        "use data.csv\n{command} y x z, offset(log_time){covariance}\ndrop y person\nreplace x = x + .2\nreplace log_time = log_time - .3\ngsort -id\npredict pm0, pmf(0)\npredict cd0, cdf(0)\npredict pm2, pmf(2)\npredict cd2, cdf(2)\npredict pm99, pmf(99)\npredict cd99, cdf(99)\nexport predictions.csv, replace\n"
      );
      fs::write(dir.path().join("fit.td"), script).unwrap();
      for mode in ["--json", "--text"] {
        let o = tabdat_bin()
          .current_dir(dir.path())
          .args([mode, "-f", "fit.td"])
          .output()
          .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        if mode == "--json" {
          for line in String::from_utf8(o.stdout).unwrap().lines() {
            let e: Value = serde_json::from_str(line).unwrap();
            assert!(e.get("error").is_none());
          }
        }
        let bytes = fs::read(dir.path().join("predictions.csv")).unwrap();
        if let Some(expected) = &baseline {
          assert_eq!(&bytes, expected);
        } else {
          baseline = Some(bytes.clone());
        }
        for (index, line) in String::from_utf8(bytes)
          .unwrap()
          .lines()
          .skip(1)
          .enumerate()
        {
          let row: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
          let id = 36 - index;
          assert_eq!(row[0], id as f64);
          for (j, k) in [0, 2, 99].into_iter().enumerate() {
            let (pmf, cdf) = refs[&(id, k)];
            for (actual, expected) in [(row[4 + 2 * j], pmf), (row[5 + 2 * j], cdf)] {
              assert!(
                (actual - expected).abs() <= expected.abs() * 1e-8 + f64::MIN_POSITIVE,
                "{family}/{id}/{k}: {actual} != {expected}"
              );
              compared += 1;
            }
          }
        }
      }
    }
  }
  assert_eq!(compared, 2592);
}
