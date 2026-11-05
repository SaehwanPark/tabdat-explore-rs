# Independent R GLM and score/delta reference; never reads native predictions.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
if (!(length(args) %in% c(2,3)) || (length(args)==3 && args[3]!='--csv'))
  stop('usage: reference.R source output-directory [--csv]')
if (file.exists(args[2])) stop('reference directory already exists')
if (length(args)==3) {
  source <- read.csv(args[1])
} else {
  tokens <- scan(args[1],what=character(),quiet=TRUE)
  stopifnot(length(tokens)==27766*12)
  raw <- matrix(tokens,ncol=12,byrow=TRUE)
  values <- matrix(as.numeric(replace(raw,raw=='.',NA)),ncol=12)
  stopifnot(identical(which(rowSums(is.na(values))>0),27734L),
            all(is.na(values[27734,])))
  source <- data.frame(source_row=seq_len(nrow(values)), values)
  names(source) <- c('source_row','pharvis','lnhhexp','age','sex','married','educ',
                     'illness','injury','illdays','actdays','insurance','commune')
}
d <- source[!is.na(source$pharvis),]
form <- pharvis ~ lnhhexp+insurance+sex+age+married+illdays+actdays+injury+illness+educ
fit <- glm(form, data=d, family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
fit <- glm(form, data=d, family=poisson(),start=coef(fit),
           control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged, max(abs(fit$weights-fitted(fit)))<1e-9)
X <- model.matrix(fit); y <- d$pharvis; b <- coef(fit); mu <- fitted(fit)
n <- length(y); g <- length(unique(d$commune)); k <- ncol(X)
B <- solve(crossprod(X,X*mu)); score <- X*(y-mu)
grouped <- rowsum(score,d$commune,reorder=FALSE)
V <- list(classical=B, robust=n/(n-1)*B%*%crossprod(score)%*%B,
          cluster=g/(g-1)*B%*%crossprod(grouped)%*%B)
ll <- sum(dpois(y,mu,log=TRUE)); ll0 <- sum(dpois(y,mean(y),log=TRUE))
rows <- list()
emit <- function(mode,quantity,i=0L,j=0L,value) {
  rows[[length(rows)+1L]] <<- data.frame(mode=mode,quantity=quantity,i=i,j=j,value=unname(value))
}
normal <- function(mode,prefix,i,value,var) {
  se <- sqrt(var); z <- value/se
  fields <- c(value=value,standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)),
              confidence_lower=value-qnorm(.975)*se,confidence_upper=value+qnorm(.975)*se)
  for (name in names(fields)) emit(mode,paste0(prefix,name),i,value=fields[[name]])
}
for (mode in names(V)) {
  cov <- V[[mode]]
  for (i in seq_len(k)) normal(mode,'coefficient_',i-1L,unname(b[i]),cov[i,i])
  for (i in seq_len(k)) for (j in seq_len(k)) emit(mode,'covariance',i-1L,j-1L,cov[i,j])
  for (i in 2:k) {
    gradient <- b[i]*colMeans(X*mu); gradient[i] <- gradient[i]+mean(mu)
    normal(mode,'effect_',i-2L,unname(b[i]*mean(mu)),drop(t(gradient)%*%cov%*%gradient))
  }
  pearson <- sum((y-mu)^2/mu)
  fields <- c(observation_count=n,zero_count=sum(y==0),count_sum=sum(y),
    log_likelihood=ll,null_log_likelihood=ll0,likelihood_ratio=max(0,2*(ll-ll0)),
    mcfadden_r_squared=1-ll/ll0,
    residual_deviance=sum(2*(ifelse(y==0,0,y*log(ifelse(y==0,1,y)/mu))-y+mu)),
    pearson_chi_square=pearson,df_residual=n-k,pearson_dispersion=pearson/(n-k))
  for (name in names(fields)) emit(mode,name,value=fields[[name]])
  if (mode!='classical') {
    w <- drop(t(b[-1])%*%solve(cov[-1,-1],b[-1]))
    emit(mode,'wald_statistic',value=w)
    emit(mode,'wald_p_value',value=pchisq(w,k-1,lower.tail=FALSE))
  }
}
dir.create(args[2])
write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(source,file.path(args[2],'source.csv'),row.names=FALSE,quote=FALSE,na='')
write.csv(data.frame(d,pooled_mu=mu,pooled_xb=drop(X%*%b)),
          file.path(args[2],'predictions.csv'),row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,';',n,'observations;',g,'communes;',length(rows),'quantities\n')
