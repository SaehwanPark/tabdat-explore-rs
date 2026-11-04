# Independent development-only base-R reference. Do not read the native CSV.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L, !file.exists(args[2]))
dir.create(args[2])
tokens <- scan(args[1], what=character(), quiet=TRUE)
stopifnot(length(tokens)>0L, length(tokens)%%25L==0L)
src <- matrix(tokens, ncol=25L, byrow=TRUE)
stopifnot(all(which(src=='.',arr.ind=TRUE)[,2]==2L))
values <- suppressWarnings(matrix(as.numeric(src), ncol=25L))
stopifnot(!anyNA(values[,-2L]), all(is.finite(values[,-2L])),
  all(values[,16:25]>=0), all(values[,16:25]==floor(values[,16:25])))
firm <- rep(seq_len(nrow(values)), each=5L)
year <- rep(1975:1979, times=nrow(values))
col <- rep(1:5, times=nrow(values))
data <- data.frame(id=firm,year=year,cusip=values[firm,1],ardssic=values[firm,2],
  scisect=values[firm,3],logk=values[firm,4],sumpat=values[firm,5],
  logr=values[cbind(firm,10L+col)],pat=values[cbind(firm,20L+col)])
stopifnot(nrow(data)==5L*nrow(values), length(unique(data$id))==nrow(values),
  !anyDuplicated(data[c('id','year')]), sum(is.na(data$ardssic))==5L*sum(is.na(values[,2])))
if (nrow(values)==346L) {
  stopifnot(sum(is.na(data$ardssic))==50L, sum(rowsum(data$pat,data$id)==0)==22L)
}
fit <- glm(pat~logr, data=data, family=poisson(),
  control=glm.control(epsilon=1e-13,maxit=100))
# Refresh IRLS weights at the converged parameter rather than the previous iterate.
fit <- glm(pat~logr, data=data, family=poisson(), start=coef(fit),
  control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged, max(abs(fit$weights-fitted(fit)))<1e-9)
X <- model.matrix(fit); y <- data$pat; b <- coef(fit); mu <- fitted(fit)
n <- length(y); g <- length(unique(data$id)); k <- ncol(X)
B <- solve(crossprod(X,X*mu))
score <- X*(y-mu); grouped <- rowsum(score,data$id,reorder=FALSE)
V <- list(classical=B, robust=n/(n-1)*B%*%crossprod(score)%*%B,
  cluster=g/(g-1)*B%*%crossprod(grouped)%*%B)
ll <- sum(dpois(y,mu,log=TRUE)); ll0 <- sum(dpois(y,mean(y),log=TRUE))
rows <- list()
emit <- function(mode,quantity,i=0L,j=0L,value) {
  rows[[length(rows)+1L]] <<- data.frame(mode=mode,quantity=quantity,i=i,j=j,value=unname(value))
}
normal <- function(mode,prefix,i,value,var) {
  value <- unname(value); se <- sqrt(var); z <- value/se
  fields <- c(value=value,standard_error=se,statistic=z,p_value=2*pnorm(-abs(z)),
    confidence_lower=value-qnorm(.975)*se,confidence_upper=value+qnorm(.975)*se)
  for (name in names(fields)) emit(mode,paste0(prefix,name),i,value=fields[[name]])
}
for (mode in names(V)) {
  cov <- V[[mode]]
  for (i in seq_len(k)) normal(mode,'coefficient_',i-1L,b[i],cov[i,i])
  for (i in seq_len(k)) for (j in seq_len(k)) emit(mode,'covariance',i-1L,j-1L,cov[i,j])
  gradient <- b[2]*colMeans(X*mu); gradient[2] <- gradient[2]+mean(mu)
  normal(mode,'effect_',0L,b[2]*mean(mu),drop(t(gradient)%*%cov%*%gradient))
  pearson <- sum((y-mu)^2/mu)
  fields <- c(observation_count=n,zero_count=sum(y==0),count_sum=sum(y),
    log_likelihood=ll,null_log_likelihood=ll0,likelihood_ratio=max(0,2*(ll-ll0)),
    mcfadden_r_squared=1-ll/ll0,
    residual_deviance=sum(2*(ifelse(y==0,0,y*log(ifelse(y==0,1,y)/mu))-y+mu)),
    pearson_chi_square=pearson,df_residual=n-k,pearson_dispersion=pearson/(n-k))
  for (name in names(fields)) emit(mode,name,value=fields[[name]])
  if (mode!='classical') {
    w <- b[2]^2/cov[2,2]
    emit(mode,'wald_statistic',value=w)
    emit(mode,'wald_p_value',value=pchisq(w,1,lower.tail=FALSE))
  }
}
write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(data,mu=mu,xb=drop(X%*%b)),file.path(args[2],'predictions.csv'),
  row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,'; 3 covariance modes;',n,'firm-years;',g,'firms;',length(rows),'quantities\n')
