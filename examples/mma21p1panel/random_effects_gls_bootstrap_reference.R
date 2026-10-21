# Independent raw-MOM fixed-theta RE-GLS transforms, Python reference indices and R lm/cov/quantile audit.
# Rscript random_effects_gls_bootstrap_reference.R MOM.dat fresh_output_directory
options(digits=17)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==2,!file.exists(args[2]))
script <- sub('^--file=','',commandArgs()[grepl('^--file=',commandArgs())])
repo <- normalizePath(file.path(dirname(script),'../..'))
if(grepl('\\.dat$',args[1],ignore.case=TRUE)) {
 v <- scan(args[1],quiet=TRUE);stopifnot(length(v)==5320*8,all(is.finite(v)))
 d <- as.data.frame(matrix(v,ncol=8,byrow=TRUE));names(d) <- c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
 d <- cbind(source_row=seq_len(nrow(d)),d)
 stopifnot(identical(d$id,rep(as.numeric(1:532),each=10)),identical(d$year,rep(as.numeric(1979:1988),532)))
} else d <- read.csv(args[1])
stopifnot(!anyDuplicated(d[c('id','year')]),!anyDuplicated(d$source_row))
d <- d[is.finite(d$lnhr)&is.finite(d$lnwg),];d <- d[order(d$source_row),]
sizes <- table(d$id);G <- length(sizes);N <- nrow(d);B <- 500;seed <- 10001
stopifnot(G>2,all(sizes==sizes[1]),sizes[1]>=2)
re <- plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='random',random.method='swar')
theta <- as.numeric(re$ercomp$theta);stopifnot(is.finite(theta),theta>=0,theta<1)
f <- d;f$mean_lnhr <- ave(f$lnhr,f$id);f$mean_lnwg <- ave(f$lnwg,f$id)
f$within_lnhr <- f$lnhr-f$mean_lnhr;f$within_lnwg <- f$lnwg-f$mean_lnwg
f$theta <- theta;f$quasi_lnhr <- f$lnhr-theta*f$mean_lnhr;f$quasi_lnwg <- f$lnwg-theta*f$mean_lnwg
members <- lapply(unique(f$id),function(key) which(f$id==key))
fit <- lm(quasi_lnhr~quasi_lnwg,f);X <- model.matrix(fit);beta <- unname(coef(fit))
stopifnot(abs(beta[1]/(1-theta)-unname(coef(re)[1]))<1e-9,abs(beta[2]-unname(coef(re)[2]))<1e-9)
index_file <- tempfile(fileext='.csv')
status <- system2('python3',c(shQuote(file.path(repo,'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py')),'--draws',seed,G,B,shQuote(index_file)))
stopifnot(status==0); indices <- read.csv(index_file,colClasses='character'); unlink(index_file)
bs <- t(vapply(indices$indices,function(text) {
  chosen <- as.integer(strsplit(text,':',fixed=TRUE)[[1]])+1
  index <- unlist(members[chosen],use.names=FALSE)
  point <- lm.fit(X[index,,drop=FALSE],f$quasi_lnhr[index]); stopifnot(point$rank==2)
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
emit('/observation_count',N); emit('/replications',B); emit('/theta',theta)
f$quasi_fitted <- fitted(fit); f$quasi_residual <- residuals(fit)
dir.create(args[2]); write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(f,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'N',N,'B',B,'seed',seed,'bootstrap SE',sqrt(diag(V)),'\n')
