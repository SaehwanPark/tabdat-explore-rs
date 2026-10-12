# Reshape fixture-directory ownership (in progress)

Base e3a6618; [issue167](https://github.com/SaehwanPark/tabdat-explore-rs/issues/167).
Unchanged root reshape tests intermittently fail reading generated Parquet, now
including line301 missing reshape_wide_collision.parquet during #179 verification.
Focused and full reruns passed; original failure cause remains unproven.

Fixture currently chooses pid+SystemTime.as_nanos then create_dir_all, which can
silently adopt another fixture's existing directory. Drop recursively deletes its
root. Wall-clock uniqueness is not ownership proof. Correct this concrete risk
without claiming it caused the observed native read failure.

Bounded plan: process-local atomic sequence for distinct candidates, exclusive
create_dir claiming, retry only AlreadyExists candidate ownership (never retry
Parquet read), preserve other I/O failures. Add deterministic existing-directory
claim rejection/marker preservation and concurrent retained fixture independence.
No dependency, runtime/semantic/tolerance/sleep change. Stress reshape suite and
full locked fmt/check/test/Clippy; leave issue open unless actual cause/reproduction
established. Do not infer all runtime helpers/backend correctness from one file.
