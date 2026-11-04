# Independent development-only original-source and bootstrap R refits.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==3L,!file.exists(args[3]))
dir.create(args[3])
tokens <- scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0L,length(tokens)%%25L==0L)
src <- matrix(tokens,ncol=25L,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
values <- suppressWarnings(matrix(as.numeric(src),ncol=25L))
stopifnot(!anyNA(values[,-2L]),all(is.finite(values[,-2L])))
xall <- values[,11:15,drop=FALSE]; yall <- values[,21:25,drop=FALSE]
stopifnot(all(is.finite(xall)),all(is.finite(yall)),all(yall>=0),
  all(yall<=1000),all(yall==floor(yall)))
keep <- rowSums(yall)>0; ids <- which(keep)
x <- xall[keep,,drop=FALSE]; y <- yall[keep,,drop=FALSE]
x <- x-rowMeans(x); total <- rowSums(y); observed <- rowSums(y*x); g <- nrow(x)
stopifnot(g>=2L,g<=346L,max(abs(x))<=20)
draws <- read.csv(args[2]); stopifnot(nrow(draws)==500L*g,
  all(draws$replicate==rep(1:500,each=g)),all(draws$position==rep(1:g,500)),
  all(draws$index0>=0 & draws$index0<g))
indices <- matrix(draws$index0+1L,nrow=500L,byrow=TRUE)
score <- function(b,mult) {
  w <- exp(b*x); sum(mult*(observed-total*rowSums(x*w)/rowSums(w)))
}
fit <- function(mult) {
  stopifnot(score(-10,mult)>0,score(10,mult)<0)
  uniroot(score,interval=c(-10,10),mult=mult,tol=1e-13)$root
}
original <- fit(rep(1,g))
beta <- vapply(1:500,function(rep)fit(tabulate(indices[rep,],nbins=g)),numeric(1))
# Independent dummy-variable Poisson fits verify the profiled score/refit route.
for (rep in c(1L,250L,500L)) {
  mult <- tabulate(indices[rep,],nbins=g); selected <- which(mult>0)
  d <- data.frame(id=rep(seq_along(selected),each=5L),
    logr=as.vector(t(x[selected,,drop=FALSE])),pat=as.vector(t(y[selected,,drop=FALSE])),
    frequency=rep(mult[selected],each=5L))
  glmfit <- glm(pat~logr+factor(id),data=d,weights=frequency,family=poisson(),
    control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(glmfit$converged,abs(unname(coef(glmfit)['logr'])-beta[rep])<1e-8)
}
fields <- c(candidate_rows=5*nrow(values),candidate_firms=nrow(values),retained_rows=5*g,
  firms_per_draw=g,replications=500,estimate=original,bootstrap_mean=mean(beta),
  bias=mean(beta)-original,bootstrap_se=sd(beta),
  ci_lower=unname(quantile(beta,.025,type=7)),ci_upper=unname(quantile(beta,.975,type=7)))
write.csv(data.frame(field=names(fields),value=unname(fields)),file.path(args[3],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(replicate=1:500,beta=beta),file.path(args[3],'replicates.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(index0=seq_along(ids)-1L,firm_id=ids),file.path(args[3],'groups.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(id=rep(seq_len(nrow(values)),each=5L),year=rep(1975:1979,nrow(values)),
  logr=as.vector(t(xall)),pat=as.vector(t(yall)),firm_total=rep(rowSums(yall),each=5L),
  included=rep(keep,each=5L)),file.path(args[3],'design.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,';',g,'firms;',length(beta),'independent conditional refits and 3 dummy-GLM cross-checks\n')
