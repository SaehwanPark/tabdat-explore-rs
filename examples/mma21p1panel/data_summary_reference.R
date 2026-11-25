# Base R reference for the source program's bounded Chapter 21 data block.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop("usage: data_summary_reference.R MOM.dat fresh_reference_directory")
input <- args[[1]]
out <- args[[2]]
if (file.exists(out) || !dir.create(out, recursive=TRUE)) stop("reference output must be a new directory")

names8 <- c("lnhr","lnwg","kids","ageh","agesq","disab","id","year")
d <- read.table(input, header=FALSE, col.names=names8, check.names=FALSE)
if (nrow(d)!=5320L || any(!is.finite(as.matrix(d)))) stop("expected 5,320 finite eight-field source records")
if (!identical(sort(unique(d$id)), as.numeric(1:532)) ||
    !identical(sort(unique(d$year)), as.numeric(1979:1988))) stop("unexpected MOM panel keys")
if (anyDuplicated(d[c("id","year")])) stop("duplicate MOM person/year key")
by_id_years <- split(d$year, d$id)
if (length(by_id_years)!=532L || any(vapply(by_id_years, function(x) length(unique(x))!=10L || !identical(sort(x),as.numeric(1979:1988)), logical(1)))) {
  stop("expected each person to have the ten consecutive 1979–1988 observations")
}

year_frequency <- as.data.frame(table(year=factor(d$year,levels=1979:1988)))
names(year_frequency) <- c("year","Count")
year_frequency$Percent <- 100*year_frequency$Count/sum(year_frequency$Count)
year_frequency$year <- as.integer(as.character(year_frequency$year))
write.csv(year_frequency,file.path(out,"year_frequency.csv"),row.names=FALSE)
variables <- c(names8,paste0("dyear",1:10))
add_year_indicators <- function(data) {
  for (i in seq_along(1979:1988)) data[[paste0("dyear",i)]] <- as.integer(data$year==1978+i)
  data
}
summary_table <- function(data) do.call(rbind,lapply(variables,function(variable) {
  x <- data[[variable]]
  data.frame(variable=variable,observations=length(x),mean=mean(x),std_dev=sd(x),
             minimum=min(x),maximum=max(x),stringsAsFactors=FALSE)
}))
d <- add_year_indicators(d)
write.csv(summary_table(d),file.path(out,"summary.csv"),row.names=FALSE,na="")

# The Stata 8 author data are stored as float32; retain a separate audit path.
to_float32 <- function(x) readBin(writeBin(as.double(x),raw(),size=4),what="double",n=length(x),size=4)
d_author <- d
for (variable in names8) d_author[[variable]] <- to_float32(d_author[[variable]])
write.csv(summary_table(d_author),file.path(out,"author_float32_summary.csv"),row.names=FALSE,na="")

ids <- sort(unique(d$id))
years <- sort(unique(d$year))
pair_counts <- table(factor(d$id,levels=ids),factor(d$year,levels=years))
row_counts <- rowSums(pair_counts)
period_counts <- rowSums(pair_counts>0)
people <- data.frame(id=ids,observations=as.integer(row_counts),periods=as.integer(period_counts),
                     first_year=integer(length(ids)),last_year=integer(length(ids)),
                     pattern=character(length(ids)))
for (j in seq_along(ids)) {
  y <- sort(d$year[d$id==ids[[j]]])
  people$first_year[[j]] <- min(y)
  people$last_year[[j]] <- max(y)
  people$pattern[[j]] <- paste(as.integer(pair_counts[j,]),collapse=",")
}
delta <- unlist(lapply(split(d$year,d$id),function(x) diff(sort(x))),use.names=FALSE)
probabilities <- c(.05,.25,.50,.75,.95)
observation_quantiles <- quantile(people$observations,probabilities,type=7,names=FALSE)
period_quantiles <- quantile(people$periods,probabilities,type=7,names=FALSE)
structure <- data.frame(observations=nrow(d),people=length(ids),first_id=min(ids),last_id=max(ids),
                        first_year=min(d$year),last_year=max(d$year),min_T=min(people$observations),
                        mean_T=mean(people$observations),max_T=max(people$observations),
                        min_periods=min(people$periods),max_periods=max(people$periods),
                        min_delta=min(delta),max_delta=max(delta),
                        observation_count=nrow(d),panel_count=length(ids),panel_min=min(ids),
                        panel_max=max(ids),period_count=length(years),time_min=min(years),
                        time_max=max(years),duplicate_key_pairs=sum(pair_counts>1),
                        extra_pair_observations=sum(pmax(pair_counts-1,0)),
                        missing_panel_periods=sum(length(years)-period_counts),
                        balanced=all(pair_counts==1),
                        observations_p5=observation_quantiles[[1]],
                        observations_p25=observation_quantiles[[2]],
                        observations_p50=observation_quantiles[[3]],
                        observations_p75=observation_quantiles[[4]],
                        observations_p95=observation_quantiles[[5]],
                        periods_p5=period_quantiles[[1]],periods_p25=period_quantiles[[2]],
                        periods_p50=period_quantiles[[3]],periods_p75=period_quantiles[[4]],
                        periods_p95=period_quantiles[[5]])
write.csv(structure,file.path(out,"panel_structure.csv"),row.names=FALSE)

freq <- aggregate(id~observations+pattern,people,length)
names(freq)[names(freq)=="id"] <- "people"
freq <- freq[order(freq$observations,freq$pattern),]
freq$percent <- 100*freq$people/sum(freq$people)
freq$cumulative_percent <- cumsum(freq$percent)
names(freq)[names(freq)=="observations"] <- "T_i"
write.csv(freq,file.path(out,"panel_t_distribution.csv"),row.names=FALSE)

# xtsum shifts within-person deviations back to the pooled mean scale.
xtsum_table <- function(data) do.call(rbind,lapply(c("lnhr","lnwg","kids","ageh","agesq","disab"),function(variable) {
  x <- data[[variable]]
  person_mean <- ave(x,data$id,FUN=mean)
  between <- aggregate(x,list(id=data$id),mean)$x
  within <- x-person_mean+mean(x)
  data.frame(variable=variable,overall_N=length(x),overall_mean=mean(x),overall_sd=sd(x),
             overall_min=min(x),overall_max=max(x),between_n=length(between),
             between_sd=sd(between),between_min=min(between),between_max=max(between),
             within_observations=length(within),mean_T=length(x)/length(between),
             within_mean=mean(within),within_sd=sd(within),within_min=min(within),
             within_max=max(within),stringsAsFactors=FALSE)
}))
write.csv(xtsum_table(d),file.path(out,"xtsum.csv"),row.names=FALSE,na="")
write.csv(xtsum_table(d_author),file.path(out,"author_float32_xtsum.csv"),row.names=FALSE,na="")
