# Synthetic independent dummy-Poisson cross-check of distinct copied group IDs.
# The main reference uses group multiplicities; this expands raw copies instead.
a <- commandArgs(trailingOnly=TRUE)
if(length(a)!=3) stop('usage: individual_conditional_fe_bootstrap_glm_check.R synthetic.csv oracle-draws.csv reference-dir')
d <- read.csv(a[1]);d <- d[!is.na(d$pharvis),];d <- d[order(d$source_row),]
vars <- c('lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ')
totals <- tapply(d$pharvis,d$commune,sum);d <- d[unname(totals[as.character(d$commune)])>0,]
ids <- unique(d$commune);G <- length(ids);stopifnot(G==18,nrow(d)==140)
draws <- read.csv(a[2]);ref <- read.csv(file.path(a[3],'fits.csv'))
stopifnot(nrow(draws)==500*G,nrow(ref)==500)
for(r in c(1,250,500)) {
  chosen <- draws$cluster_index[draws$replicate==r]+1L
  stopifnot(length(chosen)==G,length(unique(chosen))<G)
  if(r==500) stopifnot(any(table(d$commune)[as.character(ids[chosen])]==1))
  pieces <- lapply(seq_along(chosen),function(i) {
    group <- d[d$commune==ids[chosen[i]],];group$copy_id <- i;group
  })
  copied <- do.call(rbind,pieces)
  D <- model.matrix(~factor(copy_id)-1,data=copied)
  X <- cbind(D,as.matrix(copied[vars]))
  fit <- glm.fit(X,copied$pharvis,family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(fit$converged,fit$rank==ncol(X),all(is.finite(fit$coefficients)))
  actual <- tail(fit$coefficients,length(vars));expected <- as.numeric(ref[r,vars])
  diff <- max(abs(actual-expected));stopifnot(diff<=1e-9)
  cat('replicate',r,'raw-copy dummy GLM versus conditional score max slope difference',format(diff,digits=6),'\n')
}
