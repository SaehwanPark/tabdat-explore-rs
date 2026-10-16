# Development-only independent R 4.6.1 reference for pure logit contracts.
# Rscript crates/tabdat-stats/tests/fixtures/logit_reference.R
options(digits=17)
i <- 1:40
x1 <- ((i*7)%%17)/5-1.6
x2 <- ((i*11)%%19)/7-9/7
y <- as.double(((i*13)%%37)<18+4*x1-3*x2)
for(intercept in c(TRUE,FALSE)) for(two in c(TRUE,FALSE)) {
  f <- if(two) y~x1+x2 else y~x1
  if(!intercept) f <- update(f,~.-1)
  fit <- glm(f,family=binomial(),control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(fit$converged, !fit$boundary)
  X <- model.matrix(fit); b <- coef(fit); p <- as.vector(predict(fit,type="response"))
  w <- p*(1-p); V <- solve(crossprod(X,X*w)); se <- sqrt(diag(V))
  null <- if(intercept) mean(y) else .5
  ll <- as.numeric(logLik(fit)); ll0 <- sum(dbinom(y,1,null,log=TRUE))
  gradient <- function(j) {
    g <- b[j]*colMeans(X*(w*(1-2*p))); g[j] <- g[j]+mean(w); g
  }
  indices <- seq_along(b)[if(intercept) seq_along(b)>1 else rep(TRUE,length(b))]
  ame <- b[indices]*mean(w)
  ame_se <- vapply(indices,function(j) {g<-gradient(j);sqrt(drop(t(g)%*%V%*%g))},0.0)
  cat("\nintercept:",intercept," two predictors:",two,"\n")
  print(list(events=sum(y),parameters=b,covariance=V,se=se,z=b/se,
    p_value=2*pnorm(-abs(b/se)),ll=ll,ll0=ll0,lr=2*(ll-ll0),
    lr_p=pchisq(2*(ll-ll0),length(indices),lower.tail=FALSE),
    pseudo_r_squared=1-ll/ll0,probability_at_rows_1_20_40=p[c(1,20,40)],
    ame=ame,ame_se=ame_se,ame_z=ame/ame_se,
    ame_p=2*pnorm(-abs(ame/ame_se)),ame_lower95=ame-qnorm(.975)*ame_se,
    ame_upper95=ame+qnorm(.975)*ame_se))
}
