# Independent development-only source and pooled-Poisson firm-bootstrap reference.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==3L,!file.exists(args[3])); dir.create(args[3])
tokens <- scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0L,length(tokens)%%25L==0L)
src <- matrix(tokens,ncol=25L,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
v <- suppressWarnings(matrix(as.numeric(src),ncol=25L))
stopifnot(!anyNA(v[,-2L]),all(is.finite(v[,-2L])))
x <- v[,11:15,drop=FALSE]; y <- v[,21:25,drop=FALSE]; g <- nrow(x)
stopifnot(g>=2L,g<=346L,all(is.finite(x)),max(abs(x))<=20,
  all(is.finite(y)),all(y>=0),all(y<=1000),all(y==floor(y)))
draws <- read.csv(args[2]); stopifnot(nrow(draws)==500L*g,
  all(draws$replicate==rep(1:500,each=g)),all(draws$position==rep(1:g,500)),
  all(draws$index0>=0 & draws$index0<g))
indices <- matrix(draws$index0+1L,nrow=500L,byrow=TRUE)
observed <- rowSums(y*x); totals <- rowSums(y)
score <- function(b,mult) {
  w <- exp(b*x); sum(mult*observed)-sum(mult*totals)*sum(mult*rowSums(x*w))/sum(mult*rowSums(w))
}
fit <- function(mult) {
  total <- sum(mult*totals); stopifnot(total>0,score(-10,mult)>0,score(10,mult)<0)
  b <- uniroot(score,interval=c(-10,10),mult=mult,tol=1e-13)$root
  a <- log(total/sum(mult*rowSums(exp(b*x))))
  c(a,b)
}
original <- fit(rep(1,g))
betas <- t(vapply(1:500,function(r)fit(tabulate(indices[r,],nbins=g)),numeric(2)))
# Independent GLM uses frequency weights rather than the profile-score equation.
for (r in c(0L,1L,250L,500L)) {
  mult <- if(r==0L) rep(1,g) else tabulate(indices[r,],nbins=g)
  keep <- which(mult>0)
  d <- data.frame(logr=as.vector(t(x[keep,,drop=FALSE])),pat=as.vector(t(y[keep,,drop=FALSE])),
    frequency=rep(mult[keep],each=5L))
  model <- glm(pat~logr,data=d,weights=frequency,family=poisson(),
    control=glm.control(epsilon=1e-13,maxit=100))
  target <- if(r==0L) original else betas[r,]
  stopifnot(model$converged,max(abs(unname(coef(model))-target))<1e-9)
}
a <- betas[,1]; b <- betas[,2]; V <- cov(betas)
fields <- c(candidate_rows=5*g,firms_per_draw=g,replications=500,
  estimate_intercept=original[1],estimate_slope=original[2],
  mean_intercept=mean(a),mean_slope=mean(b),bias_intercept=mean(a)-original[1],bias_slope=mean(b)-original[2],
  covariance_aa=V[1,1],covariance_ab=V[1,2],covariance_bb=V[2,2],
  se_intercept=sd(a),se_slope=sd(b),lower_intercept=unname(quantile(a,.025,type=7)),
  upper_intercept=unname(quantile(a,.975,type=7)),lower_slope=unname(quantile(b,.025,type=7)),upper_slope=unname(quantile(b,.975,type=7)))
write.csv(data.frame(field=names(fields),value=unname(fields)),file.path(args[3],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(replicate=1:500,intercept=a,slope=b),file.path(args[3],'replicates.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(index0=seq_len(g)-1L,firm_id=seq_len(g)),file.path(args[3],'groups.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=rep(seq_len(g),each=5L),year=rep(1975:1979,g),logr=as.vector(t(x)),pat=as.vector(t(y))),file.path(args[3],'design.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',g,'firms;',nrow(betas),'independent two-parameter refits and four GLM cross-checks\n')
