# Independent dummy-variable GLM and conditional-likelihood audit; development only.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L, !file.exists(args[2]))
dir.create(args[2])
tokens <- scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0, length(tokens)%%25L==0L)
src <- matrix(tokens,ncol=25L,byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
values <- suppressWarnings(matrix(as.numeric(src),ncol=25L))
stopifnot(!anyNA(values[,-2L]),all(is.finite(values[,-2L])),
  all(values[,16:25]>=0), all(values[,16:25]<=1000),
  all(values[,16:25]==floor(values[,16:25])))
firm <- rep(seq_len(nrow(values)),each=5L)
year <- rep(1975:1979,times=nrow(values)); col <- rep(1:5,times=nrow(values))
d <- data.frame(id=firm,year=year,cusip=values[firm,1],ardssic=values[firm,2],
  scisect=values[firm,3],logk=values[firm,4],sumpat=values[firm,5],
  logr=values[cbind(firm,10L+col)],pat=values[cbind(firm,20L+col)])
d$included_in_fe <- ave(d$pat,d$id,FUN=sum)>0
retained <- d[d$included_in_fe,]; groups <- split(retained,retained$id)
stopifnot(length(groups)>=2L,!anyDuplicated(d[c('id','year')]))
# Unconditional Poisson with a firm dummy for every positive-total firm is an
# independent maximization route to the same profiled scalar slope/information.
fit <- glm(pat~logr+factor(id),data=retained,family=poisson(),
  control=glm.control(epsilon=1e-13,maxit=100))
# Refresh the last IRLS weights at the converged parameter.
fit <- glm(pat~logr+factor(id),data=retained,family=poisson(),start=coef(fit),
  control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged, !anyNA(coef(fit)),max(abs(fit$weights-fitted(fit)))<1e-9)
beta <- unname(coef(fit)['logr']); se <- sqrt(vcov(fit)['logr','logr'])
unit <- function(group,b) {
  x <- group$logr; y <- group$pat; z <- b*x; shifted <- z-max(z)
  p <- exp(shifted)/sum(exp(shifted)); mean_x <- sum(p*x)
  list(score=sum(y*x)-sum(y)*mean_x,
    information=sum(y)*sum(p*(x-mean_x)^2),
    conditional_ll=lgamma(sum(y)+1)-sum(lgamma(y+1))+sum(y*log(p)),
    mu=sum(y)*p)
}
conditional_score <- function(b) sum(vapply(groups,function(g)unit(g,b)$score,numeric(1)))
root <- uniroot(conditional_score,interval=c(-10,10),tol=1e-13)$root
stopifnot(abs(beta-root)<1e-9)
parts <- lapply(groups,unit,b=beta)
score <- sum(vapply(parts,`[[`,numeric(1),'score'))
information <- sum(vapply(parts,`[[`,numeric(1),'information'))
ll <- sum(vapply(parts,`[[`,numeric(1),'conditional_ll'))
stopifnot(abs(score)<1e-7, abs(se-1/sqrt(information))<1e-9,
  max(abs(fitted(fit)-unlist(lapply(parts,`[[`,'mu'),use.names=FALSE)))<1e-7)
z <- beta/se
fields <- c(candidate_rows=nrow(d),candidate_firms=nrow(values),retained_rows=nrow(retained),
  retained_firms=length(groups),excluded_rows=nrow(d)-nrow(retained),
  excluded_firms=nrow(values)-length(groups),beta=beta,se=se,z=z,
  p_value=2*pnorm(-abs(z)),ci_lower=beta-qnorm(.975)*se,ci_upper=beta+qnorm(.975)*se,
  conditional_ll=ll,information=information)
write.csv(data.frame(field=names(fields),value=unname(fields)),
  file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
d$fe_mean <- NA_real_; d$fe_residual <- NA_real_
d$fe_mean[d$included_in_fe] <- unlist(lapply(parts,`[[`,'mu'),use.names=FALSE)
d$fe_residual[d$included_in_fe] <- retained$pat-d$fe_mean[d$included_in_fe]
write.csv(d,file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,'; retained',nrow(retained),'rows,',length(groups),'firms;',length(fields),'quantities\n')
