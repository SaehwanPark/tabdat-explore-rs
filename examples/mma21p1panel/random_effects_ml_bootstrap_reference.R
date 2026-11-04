#!/usr/bin/env Rscript
# Independent dense-QR marginal-likelihood fits on copied person blocks.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript random_effects_ml_bootstrap_reference.R MOM.dat fresh_output_directory')
argv <- commandArgs(FALSE)
self <- sub('^--file=', '', argv[grepl('^--file=', argv)])
repo <- normalizePath(file.path(dirname(self),'../..'))
out <- args[2]
if (file.exists(out)) stop('refusing existing reference directory')
d <- read.table(args[1], col.names=c('lnhr','lnwg','kids','ageh','agesq','disab','id','year'))
stopifnot(nrow(d)>0,nrow(d)<=5320,all(is.finite(as.matrix(d))),!anyDuplicated(d[c('id','year')]),
  all(d$id==floor(d$id)),all(d$id>=1 & d$id<=532),all(d$year==floor(d$year)),all(d$year>=1979 & d$year<=1988))
d$source_row <- seq_len(nrow(d)); ids <- sort(unique(d$id)); G <- length(ids)
stopifnot(G>=3,G<=532,all(table(d$id)==10),all(abs(d$lnhr)<=20),all(abs(d$lnwg)<=20))
blocks <- lapply(ids,function(id) d[d$id==id,][order(d$year[d$id==id]),])
dir.create(out,recursive=FALSE)
drawfile <- file.path(out,'draws.csv')
status <- system2('python3',c(shQuote(file.path(repo,'examples/mma23p1pannonlin/conditional_bootstrap_draws.py')),
  '10001',G,'500',G,shQuote(drawfile)))
stopifnot(status==0)
draws <- read.csv(drawfile)
fit <- function(which) {
  z <- do.call(rbind,blocks[which]); n <- nrow(z); groups <- length(which)
  # Means are repeated by copied block, even if two copies have the same original ID.
  ym <- rep(vapply(blocks[which],function(b) mean(b$lnhr),0),each=10)
  xm <- rep(vapply(blocks[which],function(b) mean(b$lnwg),0),each=10)
  profile <- function(eta, details=FALSE) {
    root <- exp(-eta/2)
    X <- cbind(rep(root,n),z$lnwg-xm+root*xm)
    y <- z$lnhr-ym+root*ym
    qrfit <- lm.fit(X,y)
    if(qrfit$rank!=2) stop('rank deficient reference replicate')
    b <- unname(qrfit$coefficients)
    rss <- sum(qrfit$residuals^2)
    e <- rss/n
    if (!is.finite(e) || e<=0) stop('nonpositive reference residual variance')
    ll <- -(n*(log(2*pi)+1+log(e))+groups*eta)/2
    between <- ym-b[1]-b[2]*xm
    score <- (n*exp(-eta)*sum(between^2)/rss-groups)/2
    if(details) return(c(intercept=b[1],slope=b[2],sigma_e_squared=e,
      sigma_u_squared=e*expm1(eta)/10,eta=eta,log_likelihood=ll,profile_score=score))
    c(ll=ll,score=score)
  }
  grid <- seq(0,24,by=.5)
  values <- vapply(grid,profile,c(ll=0,score=0))
  crossings <- which(values['score',-length(grid)]>0 & values['score',-1]<=0)
  roots <- vapply(crossings,function(i) uniroot(function(x) profile(x)['score'],
    c(grid[i],grid[i+1]),tol=1e-12)$root,0)
  candidates <- c(0,roots)
  likelihoods <- vapply(candidates,function(x) profile(x)['ll'],0)
  eta <- candidates[which.max(likelihoods)]
  ans <- profile(eta,TRUE)
  derivative <- function(h) (profile(eta+h)['score']-profile(eta-h)['score'])/(2*h)
  information <- -(4*derivative(5e-5)-derivative(1e-4))/3
  ans <- c(ans,profile_information=unname(information))
  stopifnot(values['score',length(grid)]<0,values['ll',length(grid)]<ans['log_likelihood'],
    if(eta==0) ans['profile_score']<=1e-8 else abs(ans['profile_score'])<1e-8)
  # Direct 10x10 covariance likelihood audits the eigen-projection objective.
  covariance <- diag(ans['sigma_e_squared'],10)+matrix(ans['sigma_u_squared'],10,10)
  U <- chol(covariance)
  residual <- matrix(z$lnhr-ans['intercept']-ans['slope']*z$lnwg,nrow=10)
  whitened <- forwardsolve(t(U),residual)
  direct_ll <- -(n*log(2*pi)+groups*2*sum(log(diag(U)))+sum(whitened^2))/2
  if (!is.finite(direct_ll) || abs(direct_ll-ans['log_likelihood'])>=1e-8) {
    print(ans); stop(sprintf('direct covariance likelihood %.17g differs from projection %.17g',direct_ll,ans['log_likelihood']))
  }
  ans
}
results <- vector('list',501)
for (rep in 0:500) {
  which <- if(rep==0) seq_len(G) else draws$index0[draws$replicate==rep]+1
  results[[rep+1]] <- c(replicate=rep,fit(which))
}
r <- as.data.frame(do.call(rbind,results)); original <- r[1,]; b <- r[-1,]
V <- cov(b[c('intercept','slope')]); intervals <- sapply(b[c('intercept','slope')],quantile,c(.025,.975),type=7)
summary <- data.frame(replications=500,people_per_draw=G,
 estimate_intercept=original$intercept,estimate_slope=original$slope,
 mean_intercept=mean(b$intercept),mean_slope=mean(b$slope),
 bias_intercept=mean(b$intercept)-original$intercept,bias_slope=mean(b$slope)-original$slope,
 covariance_aa=V[1,1],covariance_ab=V[1,2],covariance_bb=V[2,2],
 se_intercept=sqrt(V[1,1]),se_slope=sqrt(V[2,2]),
 lower_intercept=intervals[1,1],upper_intercept=intervals[2,1],
 lower_slope=intervals[1,2],upper_slope=intervals[2,2])
d$reml_xb <- original$intercept+original$slope*d$lnwg
d$reml_residual <- d$lnhr-d$reml_xb
# write.csv internally formats numeric cells to 15 digits; preserve source doubles.
write_precise <- function(frame,path) {
  text <- as.data.frame(lapply(frame,function(x) sprintf('%.17g',x)))
  write.table(text,path,sep=',',row.names=FALSE,col.names=TRUE,quote=FALSE)
}
write_precise(r,file.path(out,'refits.csv'))
write_precise(summary,file.path(out,'summary.csv'))
write_precise(d,file.path(out,'predictions.csv'))
cat(R.version.string,'G',G,'B500; 501 dense-QR fits/direct covariance likelihood audits; SE',sqrt(diag(V)),'\n')
