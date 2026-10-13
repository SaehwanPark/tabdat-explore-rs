use crate::{Capability, Command, DataSource};
use std::collections::BTreeSet;

impl Command {
  /// Required capability families, not a request to eagerly initialize them.
  /// Availability and supported variants must still be checked by execution.
  pub fn requirements(&self) -> Vec<Capability> {
    use Capability::*;
    let mut required = BTreeSet::new();
    match self {
      Self::Help { .. }
      | Self::Status
      | Self::Exit
      | Self::Describe
      | Self::Doctor
      | Self::Set { .. }
      | Self::Panel { .. } => {}
      Self::Use { source, .. } => {
        required.extend([Data, FileIo]);
        if matches!(source, DataSource::Uri(_)) {
          required.insert(Network);
        }
      }
      Self::Run { .. } => {
        required.insert(FileIo);
      }
      Self::Save { .. } | Self::Export { .. } => {
        required.extend([Data, FileIo]);
      }
      Self::Bayes { .. } => {
        required.extend([Data, Bayesian]);
      }
      Self::BayesPrefix { command } => {
        required.extend(command.command.requirements());
        required.insert(Bayesian);
      }
      Self::By { command } => {
        required.extend(command.command.requirements());
        required.insert(Data);
      }
      Self::Spregress { command } => {
        required.extend([Data, Spatial]);
        if command.weights_file.is_some() {
          required.insert(FileIo);
        }
      }
      Self::Histogram { .. } | Self::Scatter { .. } | Self::Bar { .. } => {
        required.extend([Data, Plotting, FileIo]);
      }
      Self::BayesPlot { .. } => {
        required.extend([Bayesian, Plotting, FileIo]);
      }
      Self::Regress { .. } | Self::Lincom { .. } | Self::Test { .. } | Self::Ttest { .. } => {
        required.extend([Data, LinearStatistics]);
      }
      Self::IvRegress { .. }
      | Self::XtReg { .. }
      | Self::XtAbond { .. }
      | Self::XtLogit { .. }
      | Self::Lowess { .. }
      | Self::Did { .. }
      | Self::DrDid { .. }
      | Self::Dml { .. }
      | Self::CfRegress { .. }
      | Self::Logit { .. }
      | Self::Probit { .. }
      | Self::Poisson { .. }
      | Self::Nbreg { .. }
      | Self::Zip { .. }
      | Self::Zinb { .. }
      | Self::Qreg { .. }
      | Self::Tobit { .. }
      | Self::Heckman { .. }
      | Self::Nl { .. }
      | Self::Streg { .. }
      | Self::Lasso { .. }
      | Self::Postlasso { .. }
      | Self::Ridge { .. }
      | Self::Elasticnet { .. } => {
        required.extend([Data, ClassicalStatistics]);
      }
      Self::Cvlasso { .. } | Self::Cvridge { .. } | Self::Cvelasticnet { .. } => {
        required.extend([Data, ClassicalStatistics, FileIo]);
      }
      // Post-estimation requirements depend on the fitted family at dispatch.
      Self::Predict { .. } | Self::Estat { .. } => {
        required.insert(Data);
      }
      Self::Summarize { .. }
      | Self::SummarizeQuery { .. }
      | Self::RowFilter { .. }
      | Self::Datasignature
      | Self::Codebook { .. }
      | Self::Missing { .. }
      | Self::Duplicates { .. }
      | Self::Isid { .. }
      | Self::Assert { .. }
      | Self::Generate { .. }
      | Self::Replace { .. }
      | Self::Keep { .. }
      | Self::Drop { .. }
      | Self::Select { .. }
      | Self::Sort { .. }
      | Self::Gsort { .. }
      | Self::Recode { .. }
      | Self::Encode { .. }
      | Self::Decode { .. }
      | Self::Label { .. }
      | Self::Tabulate { .. }
      | Self::Join { .. }
      | Self::Append { .. }
      | Self::Reshape { .. }
      | Self::XtData { .. }
      | Self::Collapse { .. }
      | Self::Rename { .. }
      | Self::Count
      | Self::Head { .. }
      | Self::Tail { .. }
      | Self::Sql { .. } => {
        required.insert(Data);
      }
    }
    required.into_iter().collect()
  }
}
