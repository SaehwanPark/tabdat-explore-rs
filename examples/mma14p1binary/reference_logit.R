# Development-only R 4.6.1 stats::glm reference, no production R dependency.
# Source specification: mma14p1binary.do, C&T (2005) section 14.2, Table14.2.
# Rscript examples/mma14p1binary/reference_logit.R Nldata.csv [predictions.csv]
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) %in% 1:2)
d <- read.csv(args[[1]])
stopifnot(nrow(d)==1182L, identical(d$source_row,seq_len(1182L)))
d <- d[d$mode %in% c(2,4), ]; d <- d[order(d$source_row), ]
d$charter <- as.integer(d$mode==4)
d$lnrelp <- log(d$pcharter/d$ppier)
stopifnot(nrow(d)==630L, sum(d$charter)==452L,
  sum(d$charter==0)==178L, all(is.finite(d$lnrelp)))
fit <- glm(charter~lnrelp,data=d,family=binomial(link="logit"),
  control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged, !fit$boundary)
X <- model.matrix(fit); b <- coef(fit); p <- as.vector(predict(fit,type="response"))
w <- p*(1-p)
# Reevaluate Fisher information at the returned coefficients, independent of
# any last-iteration working-weight lag in glm's cached QR covariance.
V <- solve(crossprod(X,X*w))
se <- sqrt(diag(V)); z <- b/se
critical <- qnorm(.975)
ll <- as.numeric(logLik(fit)); null_ll <- sum(dbinom(d$charter,1,mean(d$charter),log=TRUE))
# Conditional-on-observed-X continuous average partial derivative and delta SE.
ame <- b[2]*mean(w)
g <- b[2]*colMeans(X*(w*(1-2*p))); g[2] <- g[2]+mean(w)
ame_se <- sqrt(drop(t(g)%*%V%*%g))
stopifnot(identical(as.vector(round(b,3)),c(2.053,-1.823)))
print(list(observations=nrow(d),events=sum(d$charter),parameters=b,covariance=V,
  standard_errors=se,z=z,p_value=2*pnorm(-abs(z)),
  lower95=b-critical*se,upper95=b+critical*se,log_likelihood=ll,
  null_log_likelihood=null_ll,likelihood_ratio=2*(ll-null_ll),
  lr_p_value=pchisq(2*(ll-null_ll),1,lower.tail=FALSE),
  mcfadden_r_squared=1-ll/null_ll,probability_summary=c(mean(p),min(p),max(p)),
  continuous_ame=ame,ame_gradient=g,ame_se=ame_se,
  ame_z=ame/ame_se,ame_p_value=2*pnorm(-abs(ame/ame_se)),
  ame_lower95=ame-critical*ame_se,ame_upper95=ame+critical*ame_se),digits=17)
if(length(args)==2) write.csv(data.frame(source_row=d$source_row,charter=d$charter,
  lnrelp=d$lnrelp,probability=p),args[[2]],row.names=FALSE)
