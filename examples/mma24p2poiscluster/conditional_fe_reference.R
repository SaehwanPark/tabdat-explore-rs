# Independent dummy-Poisson GLM, profiled conditional score/information/likelihood.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (!(length(a)%in%c(2,3)) || (length(a)==3 && a[3]!='--csv'))
  stop('usage: conditional_fe_reference.R source output-directory [--csv]')
if (file.exists(a[2])) stop('reference directory already exists')
if (length(a)==3) {
  source <- read.csv(a[1])
} else {
  tokens <- scan(a[1],what=character(),quiet=TRUE)
  stopifnot(length(tokens)==27766*12)
  raw <- matrix(tokens,ncol=12,byrow=TRUE)
  v <- matrix(as.numeric(replace(raw,raw=='.',NA)),ncol=12)
  stopifnot(identical(which(rowSums(is.na(v))>0),27734L),all(is.na(v[27734,])))
  source <- data.frame(source_row=seq_len(nrow(v)),v)
  names(source) <- c('source_row','pharvis','lnhhexp','age','sex','married','educ',
                     'illness','injury','illdays','actdays','insurance','commune')
}
predictors <- c('lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ')
d <- source[complete.cases(source[,c('pharvis',predictors,'commune')]),]
d <- d[order(d$source_row),]
stopifnot(!anyDuplicated(d$source_row),all(is.finite(as.matrix(d))),all(d$pharvis>=0),
          all(d$pharvis==floor(d$pharvis)))
d$selected_commune_total <- ave(d$pharvis,d$commune,FUN=sum)
d$included_in_fe <- d$selected_commune_total>0
retained <- d[d$included_in_fe,]
groups <- split(seq_len(nrow(retained)),retained$commune)
K <- length(predictors); G <- length(groups)
stopifnot(G>K)
f <- reformulate(c(predictors,'factor(commune)'),response='pharvis')
fit <- glm(f,data=retained,family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
fit <- glm(f,data=retained,family=poisson(),start=coef(fit),control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged,!anyNA(coef(fit)),max(abs(fit$weights-fitted(fit)))<1e-9)
b <- coef(fit)[predictors]; X <- as.matrix(retained[,predictors]); y <- retained$pharvis
Vglm <- vcov(fit)[predictors,predictors]; info <- matrix(0,K,K); score <- rep(0,K)
mu <- rep(0,length(y)); ll <- 0; ll0 <- 0
for (indices in groups) {
  x <- X[indices,,drop=FALSE]; counts <- y[indices]; total <- sum(counts)
  # Independent likelihood uses max-shifted raw indices, not native anchor/scales.
  eta <- drop(x%*%b); z <- eta-max(eta); logp <- z-log(sum(exp(z))); p <- exp(logp)
  mu[indices] <- total*p
  center <- colSums(x*p); centered <- sweep(x,2,center)
  info <- info+total*crossprod(centered,centered*p)
  score <- score+colSums(centered*(counts-total*p))
  constant <- lgamma(total+1)-sum(lgamma(counts+1))
  ll <- ll+constant+sum(counts*logp)
  ll0 <- ll0+constant-total*log(length(indices))
}
V <- solve(info)
stopifnot(max(abs(score))/G<1e-9,max(abs(V-Vglm))<1e-9,
          max(abs(mu-fitted(fit)))<1e-9)
w <- drop(t(b)%*%solve(V,b)); lr <- 2*(ll-ll0)
refs <- list()
emit <- function(path,value) refs[[length(refs)+1L]] <<- data.frame(path=path,value=unname(value))
fields <- c(observation_count=nrow(retained),joint_finite_observation_count=nrow(d),
  candidate_group_count=length(unique(d$commune)),group_count=G,
  zero_total_group_count=length(unique(d$commune[!d$included_in_fe])),
  zero_total_observation_count=sum(!d$included_in_fe),minimum_group_size=min(lengths(groups)),
  maximum_group_size=max(lengths(groups)),zero_count=sum(y==0),count_sum=sum(y),
  conditional_log_likelihood=ll,null_conditional_log_likelihood=ll0,
  likelihood_ratio=lr,likelihood_ratio_df=K,likelihood_ratio_p_value=pchisq(lr,K,lower.tail=FALSE),
  wald_statistic=w,wald_p_value=pchisq(w,K,lower.tail=FALSE))
for (name in names(fields)) emit(paste0('/',name),fields[name])
emit('/sample/total_observations',nrow(d))
emit('/sample/dropped_observations',nrow(d)-nrow(retained))
for(i in seq_len(K)) {
  se <- sqrt(V[i,i]); z <- b[i]/se
  fields <- c(value=b[i],standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)),
              confidence_lower=b[i]-qnorm(.975)*se,confidence_upper=b[i]+qnorm(.975)*se)
  names(fields) <- c('value','standard_error','statistic','p_value','confidence_lower','confidence_upper')
  for (name in names(fields)) emit(sprintf('/coefficients/%d/%s',i-1,name),fields[name])
  for(j in seq_len(K)) emit(sprintf('/covariance_matrix/%d/%d',i-1,j-1),V[i,j])
}
d$fe_mean <- NA_real_; d$fe_residual <- NA_real_
d$fe_mean[d$included_in_fe] <- mu
d$fe_residual[d$included_in_fe] <- y-mu
d$fe_xb <- drop(as.matrix(d[,predictors])%*%b)
# Match the script's export column order; the auditor also checks names.
d <- d[,c(names(source),'selected_commune_total','included_in_fe','fe_mean','fe_xb','fe_residual')]
dir.create(a[2])
write.csv(source,file.path(a[2],'source.csv'),row.names=FALSE,quote=FALSE,na='')
write.csv(d,file.path(a[2],'predictions.csv'),row.names=FALSE,quote=FALSE,na='')
write.csv(do.call(rbind,refs),file.path(a[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(parameter=predictors,value=unname(b),standard_error=sqrt(diag(V))),
          file.path(a[2],'coefficients.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'; dummy-Poisson GLM / conditional information;',nrow(retained),'rows;',G,'groups;',length(refs),'quantities\n')
