# Independent raw-MOM means, Python reference indices and R lm/cov/quantile audit.
# Rscript between_bootstrap_reference.R MOM.dat fresh_output_directory
options(digits=17)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==2,!file.exists(args[2]))
script <- sub('^--file=','',commandArgs()[grepl('^--file=',commandArgs())])
repo <- normalizePath(file.path(dirname(script),'../..'))
v <- scan(args[1],quiet=TRUE); stopifnot(length(v)==5320*8,all(is.finite(v)))
d <- as.data.frame(matrix(v,ncol=8,byrow=TRUE)); names(d) <- c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
stopifnot(identical(d$id,rep(as.numeric(1:532),each=10)),identical(d$year,rep(as.numeric(1979:1988),532)))
f <- do.call(rbind,lapply(split(d,d$id),function(p) data.frame(id=p$id[1],model_rows=nrow(p),first_year=min(p$year),last_year=max(p$year),mean_lnhr=mean(p$lnhr),mean_lnwg=mean(p$lnwg))))
f <- f[order(f$id),]; N <- nrow(f); B <- 200; seed <- 10001
fit <- lm(mean_lnhr~mean_lnwg,f); X <- model.matrix(fit); beta <- unname(coef(fit))
index_file <- tempfile(fileext='.csv')
status <- system2('python3',c(shQuote(file.path(repo,'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py')),'--draws',seed,N,B,shQuote(index_file)))
stopifnot(status==0); indices <- read.csv(index_file,colClasses='character'); unlink(index_file)
bs <- t(vapply(indices$indices,function(text) {
  index <- as.integer(strsplit(text,':',fixed=TRUE)[[1]])+1
  point <- lm.fit(X[index,,drop=FALSE],f$mean_lnhr[index]); stopifnot(point$rank==2)
  unname(point$coefficients)
},numeric(2)))
V <- cov(bs); means <- colMeans(bs); rows <- list()
emit <- function(path,value) rows[[length(rows)+1]] <<- data.frame(path,value=unname(value))
for(i in 1:2) {
  ci <- quantile(bs[,i],c(.025,.975),type=7,names=FALSE)
  vals <- c(estimate=beta[i],bootstrap_mean=means[i],bias=means[i]-beta[i],standard_error=sqrt(V[i,i]),ci_lower=ci[1],ci_upper=ci[2])
  for(n in names(vals)) emit(paste0('/coefficients/',i-1,'/',n),vals[[n]])
  for(j in 1:2) emit(paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
emit('/observation_count',N); emit('/replications',B)
f$between_fitted <- fitted(fit); f$between_residual <- residuals(fit)
dir.create(args[2]); write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(f,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'N',N,'B',B,'seed',seed,'bootstrap SE',sqrt(diag(V)),'\n')
