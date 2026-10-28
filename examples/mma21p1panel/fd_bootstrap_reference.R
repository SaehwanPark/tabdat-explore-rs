# Independent raw-MOM annual differences, Python reference draws and R lm/cov/quantile audit.
# Rscript fd_bootstrap_reference.R MOM.dat fresh_output_directory
options(digits=17)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==2,!file.exists(args[2]))
script <- sub('^--file=','',commandArgs()[grepl('^--file=',commandArgs())])
repo <- normalizePath(file.path(dirname(script),'../..'))
v <- scan(args[1],quiet=TRUE); stopifnot(length(v)==5320*8,all(is.finite(v)))
d <- as.data.frame(matrix(v,ncol=8,byrow=TRUE)); names(d) <- c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
stopifnot(identical(d$id,rep(as.numeric(1:532),each=10)),identical(d$year,rep(as.numeric(1979:1988),532)))
index <- which(d$year>1979); previous <- index-1
stopifnot(all(d$id[index]==d$id[previous]),all(d$year[index]==d$year[previous]+1))
f <- data.frame(source_row=index,id=d$id[index],year=d$year[index],lnhr=d$lnhr[index],lnwg=d$lnwg[index],
 previous_source_row=previous,previous_year=d$year[previous],dlnhr=d$lnhr[index]-d$lnhr[previous],dlnwg=d$lnwg[index]-d$lnwg[previous])
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
cat(R.version.string,'N',N,'G',G,'B',B,'seed',seed,'bootstrap SE',sqrt(diag(V)),'\n')
