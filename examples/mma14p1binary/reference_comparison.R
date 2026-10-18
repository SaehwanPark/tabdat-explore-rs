# Development-only base R 4.6.1 reference; not a production dependency.
# Rscript reference_comparison.R Nldata.csv R_predictions.csv > R_comparison.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)
d <- read.csv(args[[1]])
stopifnot(nrow(d)==1182L, identical(d$source_row,seq_len(1182L)))
d <- d[d$mode %in% c(2,4), ]; d <- d[order(d$source_row), ]
d$charter <- as.integer(d$mode==4); d$lnrelp <- log(d$pcharter/d$ppier)
stopifnot(nrow(d)==630L, sum(d$charter)==452L,
  all(is.finite(d$lnrelp)), all(is.finite(d$charter)))
X <- cbind(1,d$lnrelp); y <- d$charter; n <- nrow(X); k <- ncol(X)
predictions <- d[c("source_row","mode","charter","pcharter","ppier","lnrelp")]
rows <- list()
emit <- function(family, mode, quantity, i, j, value) {
  rows[[length(rows)+1L]] <<- data.frame(family=family,mode=mode,
    quantity=quantity,i=i-1L,j=j-1L,value=value)
}
for(mode in c("classical","robust")) for(family in c("logit","probit","ols")) {
  if(family=="ols") {
    fit <- lm(charter~lnrelp,d); b <- coef(fit)
    B <- solve(crossprod(X)); scores <- X*resid(fit)
    V <- if(mode=="classical") vcov(fit) else n/(n-k)*B%*%crossprod(scores)%*%B
    ll <- as.numeric(logLik(fit))
    emit(family,mode,"r_squared",1,1,summary(fit)$r.squared)
    emit(family,mode,"root_mse",1,1,summary(fit)$sigma)
    emit(family,mode,"df_residual",1,1,df.residual(fit))
    predictions[[paste(family,mode,"xb",sep="_")]] <- drop(X%*%b)
  } else {
    fit <- glm(charter~lnrelp,d,family=binomial(link=family),
      control=glm.control(epsilon=1e-13,maxit=100))
    stopifnot(fit$converged, !fit$boundary)
    b <- coef(fit)
    parts <- function(b) {
      z <- drop(X%*%b)
      if(family=="logit") {
        p <- plogis(z); score <- y-p; curvature <- p*(1-p)
      } else {
        p <- pnorm(z); a <- dnorm(z)/(p*(1-p)); score <- (y-p)*a
        curvature <- dnorm(z)*a + score*(z+(1-2*p)*a)
      }
      list(p=p,score=score,information=crossprod(X,X*curvature))
    }
    # glm's cached probit iterate need not be a score fixed point. Reevaluate
    # observed information at the polished coefficients, not cached QR weights.
    for(it in 1:20) {
      c <- parts(b); gradient <- drop(crossprod(X,c$score))
      if(max(abs(gradient))<1e-12) break
      b <- b+solve(c$information,gradient)
    }
    c <- parts(b); stopifnot(max(abs(crossprod(X,c$score)))<1e-12)
    B <- solve(c$information); scores <- X*c$score
    V <- if(mode=="classical") B else n/(n-1)*B%*%crossprod(scores)%*%B
    ll <- sum(y*log(c$p)+(1-y)*log1p(-c$p))
    null_ll <- sum(y*log(mean(y))+(1-y)*log1p(-mean(y)))
    emit(family,mode,"mcfadden_r_squared",1,1,1-ll/null_ll)
    predictions[[paste(family,mode,"pr",sep="_")]] <- c$p
    predictions[[paste(family,mode,"xb",sep="_")]] <- drop(X%*%b)
  }
  se <- sqrt(diag(V)); statistic <- b/se
  p <- if(family=="ols") 2*pt(-abs(statistic),n-k) else 2*pnorm(-abs(statistic))
  emit(family,mode,"observation_count",1,1,n)
  emit(family,mode,"log_likelihood",1,1,ll)
  for(i in seq_len(k)) {
    emit(family,mode,"coefficient",i,1,b[i])
    emit(family,mode,"standard_error",i,1,se[i])
    emit(family,mode,"statistic",i,1,statistic[i])
    emit(family,mode,"p_value",i,1,p[i])
    for(j in seq_len(k)) emit(family,mode,"covariance",i,j,V[i,j])
  }
}
stopifnot(min(predictions$ols_classical_xb)<0, max(predictions$ols_classical_xb)>1)
write.csv(predictions,args[[2]],row.names=FALSE)
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
