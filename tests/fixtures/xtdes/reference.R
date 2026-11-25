args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop("usage: reference.R uneven.csv fresh_output_directory")
input <- args[[1]]
out <- args[[2]]
if (file.exists(out) || !dir.create(out,recursive=TRUE)) stop("output directory must be new")
d <- read.csv(input,stringsAsFactors=FALSE,colClasses=c("character","numeric"))
if (anyNA(d$firm) || any(!is.finite(d$year))) stop("invalid panel/time key")
ids <- sort(unique(d$firm))
times <- sort(unique(d$year))
counts <- matrix(0L,nrow=length(ids),ncol=length(times),
                 dimnames=list(ids,as.character(times)))
for (row in seq_len(nrow(d))) {
  i <- match(d$firm[[row]],ids)
  t <- match(d$year[[row]],times)
  counts[i,t] <- counts[i,t]+1L
}
observations <- rowSums(counts)
periods <- rowSums(counts>0)
quantiles <- function(x) quantile(x,c(.05,.25,.50,.75,.95),type=7,names=FALSE)
obs_q <- quantiles(observations)
period_q <- quantiles(periods)
structure <- data.frame(
  observation_count=nrow(d),panel_count=length(ids),panel_min=min(ids),panel_max=max(ids),
  period_count=length(times),time_min=min(times),time_max=max(times),
  duplicate_key_pairs=sum(counts>1),extra_pair_observations=sum(pmax(counts-1L,0L)),
  missing_panel_periods=sum(length(times)-periods),balanced=all(counts==1),
  observations_min=min(observations),observations_p5=obs_q[[1]],
  observations_p25=obs_q[[2]],observations_p50=obs_q[[3]],
  observations_p75=obs_q[[4]],observations_p95=obs_q[[5]],
  observations_max=max(observations),periods_min=min(periods),periods_p5=period_q[[1]],
  periods_p25=period_q[[2]],periods_p50=period_q[[3]],periods_p75=period_q[[4]],
  periods_p95=period_q[[5]],periods_max=max(periods))
write.csv(structure,file.path(out,"structure.csv"),row.names=FALSE)
write.csv(data.frame(time=times),file.path(out,"time_values.csv"),row.names=FALSE)

signatures <- apply(counts,1,paste,collapse=",")
unique_signatures <- unique(signatures)
pattern_counts <- do.call(rbind,lapply(unique_signatures,function(signature) {
  as.integer(strsplit(signature,",",fixed=TRUE)[[1]])
}))
pattern_panels <- vapply(unique_signatures,function(signature) sum(signatures==signature),integer(1))
order_args <- c(list(-pattern_panels),lapply(seq_len(ncol(pattern_counts)),function(j) pattern_counts[,j]))
ord <- do.call(order,c(order_args,list(method="radix")))
pattern_counts <- pattern_counts[ord,,drop=FALSE]
pattern_panels <- pattern_panels[ord]
pattern_percent <- 100*pattern_panels/length(ids)
pattern_cumulative <- 100*cumsum(pattern_panels)/length(ids)
if (length(pattern_panels)>0) pattern_cumulative[[length(pattern_panels)]] <- 100
patterns <- data.frame(
  counts_by_period=apply(pattern_counts,1,paste,collapse=","),
  panel_count=pattern_panels,percent=pattern_percent,cumulative_percent=pattern_cumulative,
  periods_observed=rowSums(pattern_counts>0),observations_per_panel=rowSums(pattern_counts))
write.csv(patterns,file.path(out,"patterns.csv"),row.names=FALSE)
