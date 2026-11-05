# Independent R/plm and full-dummy/weighted-between/quasi-QR reference.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (!(length(a)%in%c(2,3)) || (length(a)==3 && a[3]!='--csv'))
  stop('usage: household_re_gls_reference.R source output-directory [--csv]')
if (file.exists(a[2])) stop('reference directory already exists')
if (length(a)==3) {
  source <- read.csv(a[1])
} else {
  tokens <- scan(a[1],what=character(),quote='"',quiet=TRUE)
  stopifnot(length(tokens)==5999*8)
  raw <- matrix(tokens,ncol=8,byrow=TRUE)
  farms <- c('non farm'=0,'farm'=1)
  education <- c('Never'=0,'Nhatre'=1,'<cap I'=2,'Cap I'=3,'Cap II'=4,
                 'Cap III'=5,'Nghe SC'=6,'THCN'=7,'DHDC'=8,'DHCD'=9,
                 'Thac sy'=10,'PTS'=11,'TS'=12)
  num <- function(x) as.numeric(replace(x,x=='.',NA))
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=num(raw[,1]),
      age=num(raw[,2]),sex=num(raw[,3]),hhsize=num(raw[,4]),farm=unname(farms[raw[,5]]),
      educ=unname(education[raw[,6]]),lnhhexp=num(raw[,7]),commune=num(raw[,8]))
  stopifnot(!anyNA(source[,-2]),sum(is.na(source$lnexp12m))==993)
}
d <- source[!is.na(source$lnexp12m),]
f <- lnexp12m~lnhhexp+age+sex+hhsize+farm+educ
fit <- plm::plm(f,data=d,index=c('commune','source_row'),model='random',random.method='swar')
y <- d$lnexp12m; X <- model.matrix(f,d); ids <- sort(unique(d$commune)); g <- length(ids)
N <- nrow(X); K <- ncol(X); groups <- factor(d$commune,levels=ids); sizes <- as.numeric(table(groups))
stopifnot(g>K,!anyDuplicated(d$source_row))
D <- sapply(seq_len(g-1),function(i) as.numeric(d$commune==ids[i])-
              sizes[i]/sizes[g]*as.numeric(d$commune==ids[g]))
within <- lm.fit(cbind(X,D),y);stopifnot(within$rank==ncol(X)+ncol(D))
e <- sum(within$residuals^2)/within$df.residual
Z <- rowsum(X,groups)/sizes; Ybar <- as.numeric(rowsum(y,groups))/sizes
between <- lm.wfit(Z,Ybar,sizes);stopifnot(between$rank==K)
trace <- sum(diag(solve(crossprod(Z,Z*sizes),crossprod(Z,Z*sizes^2))))
raw_u <- (sum(sizes*between$residuals^2)-(g-K)*e)/(N-trace);u <- max(0,raw_u)
stopifnot(max(abs(c(e,u)-unname(fit$ercomp$sigma2)))<1e-9)
theta_g <- 1-sqrt(e/(e+sizes*u)); theta <- theta_g[match(d$commune,ids)]
Xmean <- Z[match(d$commune,ids),]; ymean <- Ybar[match(d$commune,ids)]
Xq <- X-theta*Xmean; yq <- y-theta*ymean
qfit <- lm.fit(Xq,yq);stopifnot(qfit$rank==K)
b <- unname(qfit$coefficients);rss <- sum(qfit$residuals^2);df <- N-K
V <- rss/df*chol2inv(qr.R(qfit$qr))
stopifnot(max(abs(b-unname(coef(fit))))<1e-9,max(abs(V-unname(vcov(fit))))<1e-9)
w <- drop(t(b[-1])%*%solve(V[-1,-1],b[-1])); refs <- list()
emit <- function(path,value) {refs[[length(refs)+1L]] <<- data.frame(path=path,value=unname(value))}
for (name in c('observation_count','group_count','minimum_group_size','maximum_group_size',
               'df_residual','sigma_e_squared','sigma_u_squared','raw_sigma_u_squared','rho',
               'theta_min','theta_max','theta_at_minimum_group_size','transformed_rss',
               'transformed_root_mse','wald_statistic','wald_df','wald_p_value')) {
  v <- switch(name,observation_count=N,group_count=g,minimum_group_size=min(sizes),
    maximum_group_size=max(sizes),df_residual=df,sigma_e_squared=e,sigma_u_squared=u,
    raw_sigma_u_squared=raw_u,rho=u/(e+u),theta_min=min(theta_g),theta_max=max(theta_g),
    theta_at_minimum_group_size=1-sqrt(e/(e+min(sizes)*u)),transformed_rss=rss,
    transformed_root_mse=sqrt(rss/df),wald_statistic=w,wald_df=K-1,
    wald_p_value=pchisq(w,K-1,lower.tail=FALSE))
  emit(paste0('/',name),v)
}
for(i in seq_len(K)) {
  se <- sqrt(V[i,i]);z <- b[i]/se
  for(name in c('value','standard_error','statistic','p_value','ci_lower','ci_upper'))
    emit(sprintf('/coefficients/%d/%s',i-1,name),switch(name,value=b[i],standard_error=se,
      statistic=z,p_value=2*pnorm(-abs(z)),ci_lower=b[i]-qnorm(.975)*se,ci_upper=b[i]+qnorm(.975)*se))
  for(j in seq_len(K)) emit(sprintf('/covariance_matrix/%d/%d',i-1,j-1),V[i,j])
}
# Separate author-output hypothesis: equal-commune between MSE and harmonic correction.
# It is not the native trace-corrected component or native retained model.
author_between <- lm.fit(Z,Ybar)
author_u <- max(0,sum(author_between$residuals^2)/(g-K)-e*mean(1/sizes))
author_theta <- (1-sqrt(e/(e+sizes*author_u)))[match(d$commune,ids)]
AX <- X-author_theta*Xmean; Ay <- y-author_theta*ymean
afit <- lm.fit(AX,Ay); A <- sum(afit$residuals^2)/(N-K)*chol2inv(qr.R(afit$qr))
author <- data.frame(parameter=c('intercept','lnhhexp','age','sex','hhsize','farm','educ'),
                     value=unname(afit$coefficients),standard_error=sqrt(diag(A)))
dir.create(a[2])
write.csv(author,file.path(a[2],'author_hypothesis.csv'),row.names=FALSE,quote=FALSE)
write.csv(do.call(rbind,refs),file.path(a[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(source,file.path(a[2],'source.csv'),row.names=FALSE,quote=FALSE,na='')
d$re_xb <- drop(X%*%b);d$re_residual <- y-d$re_xb
write.csv(d,file.path(a[2],'predictions.csv'),row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,'; plm',as.character(packageVersion('plm')),';',N,'observations;',g,'communes;',length(refs),'quantities\n')
