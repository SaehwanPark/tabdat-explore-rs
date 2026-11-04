# Independent synthetic irregular annual differences, Python reference draws and R lm/cov/quantile audit.
# Rscript fd_bootstrap_synthetic_reference.R synthetic-input.csv fresh_output_directory
options(digits=17)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==2,!file.exists(args[2]))
script <- sub('^--file=','',commandArgs()[grepl('^--file=',commandArgs())])
repo <- normalizePath(file.path(dirname(script),'../..'))
d <- read.csv(args[1],check.names=FALSE); stopifnot(nrow(d)==27)
d <- d[order(d$id,d$year),]
index <- which(seq_len(nrow(d))>1 & d$id==c(NA,head(d$id,-1)) & d$year==c(NA,head(d$year,-1))+1)
previous <- index-1
f <- data.frame(source_row=d$source_row[index],id=d$id[index],year=d$year[index],lnhr=d$lnhr[index],lnwg=d$lnwg[index],
 previous_source_row=d$source_row[previous],previous_year=d$year[previous],dlnhr=d$lnhr[index]-d$lnhr[previous],dlnwg=d$lnwg[index]-d$lnwg[previous])
stopifnot(nrow(f)==19); f <- f[is.finite(f$dlnhr) & is.finite(f$dlnwg),]; stopifnot(nrow(f)==14,length(unique(f$id))==6)
N <- nrow(f); G <- length(unique(f$id)); B <- 500; seed <- 10001
members <- lapply(unique(f$id),function(key) which(f$id==key))
fit <- lm(dlnhr~dlnwg,f); X <- model.matrix(fit); beta <- unname(coef(fit))
index_file <- tempfile(fileext='.csv')
status <- system2('python3',c(shQuote(file.path(repo,'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py')),'--draws',seed,G,B,shQuote(index_file)))
stopifnot(status==0); indices <- read.csv(index_file,colClasses='character'); unlink(index_file)
bs <- t(vapply(indices$indices,function(text) {
  chosen <- as.integer(strsplit(text,':',fixed=TRUE)[[1]])+1
  index <- unlist(members[chosen],use.names=FALSE)
  point <- lm.fit(X[index,,drop=FALSE],f$dlnhr[index]); stopifnot(point$rank==2)
  unname(point$coefficients)
},numeric(2)))
replicate_n <- vapply(indices$indices,function(text) {
 chosen <- as.integer(strsplit(text,':',fixed=TRUE)[[1]])+1
 sum(lengths(members[chosen]))
},integer(1))
V <- cov(bs); means <- colMeans(bs); rows <- list()
emit <- function(path,value) rows[[length(rows)+1]] <<- data.frame(path,value=unname(value))
for(i in 1:2) {
  ci <- quantile(bs[,i],c(.025,.975),type=7,names=FALSE)
  vals <- c(estimate=beta[i],bootstrap_mean=means[i],bias=means[i]-beta[i],standard_error=sqrt(V[i,i]),ci_lower=ci[1],ci_upper=ci[2])
  for(n in names(vals)) emit(paste0('/coefficients/',i-1,'/',n),vals[[n]])
  for(j in 1:2) emit(paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
emit('/observation_count',N); emit('/replications',B)
f$fd_fitted <- fitted(fit); f$fd_residual <- residuals(fit)
dir.create(args[2]); write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(f,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(n=replicate_n),file.path(args[2],'replicate_n.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'N',N,'G',G,'B',B,'seed',seed,'bootstrap SE',sqrt(diag(V)),'\n')
