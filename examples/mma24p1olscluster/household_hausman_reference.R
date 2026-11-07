# Independent within/full-dummy and quasi-GLS QR, separate native scales.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (!(length(a) %in% c(2,3)) || (length(a)==3 && !a[3] %in% c('--csv','--dta')))
  stop('usage: household_hausman_reference.R source new-output-directory [--csv|--dta]')
if(file.exists(a[2])) stop('reference directory already exists')
mode <- if(length(a)==3) a[3] else '--ascii'
if(mode=='--csv') {
  source <- read.csv(a[1])
} else if(mode=='--dta') {
  d <- foreign::read.dta(a[1],convert.factors=FALSE)
  source <- data.frame(source_row=seq_len(nrow(d)),lnexp12m=d$lhhex12m,age=d$age,
    sex=d$sex,hhsize=d$hhsize,farm=d$farm,educ=d$comped98,lnhhexp=d$lhhexp1,commune=d$commune)
} else {
  tokens <- scan(a[1],what=character(),quote='"',quiet=TRUE)
  stopifnot(length(tokens)==5999*8)
  raw <- matrix(tokens,ncol=8,byrow=TRUE)
  farms <- c('non farm'=0,'farm'=1)
  education <- c('Never'=0,'Nhatre'=1,'<cap I'=2,'Cap I'=3,'Cap II'=4,'Cap III'=5,
    'Nghe SC'=6,'THCN'=7,'DHDC'=8,'DHCD'=9,'Thac sy'=10,'PTS'=11,'TS'=12)
  num <- function(x) as.numeric(replace(x,x=='.',NA))
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=num(raw[,1]),age=num(raw[,2]),
    sex=num(raw[,3]),hhsize=num(raw[,4]),farm=unname(farms[raw[,5]]),educ=unname(education[raw[,6]]),
    lnhhexp=num(raw[,7]),commune=num(raw[,8]))
}
stopifnot(!anyNA(source[,-2]),!anyDuplicated(source$source_row))
d <- source[!is.na(source$lnexp12m),]; d <- d[order(d$source_row),]
X <- cbind(1,d$lnhhexp,d$age,d$sex,d$hhsize,d$farm,d$educ); y <- d$lnexp12m
N <- nrow(d); K <- ncol(X); group <- factor(d$commune); sizes <- as.numeric(table(group)); G <- length(sizes)
Z <- rowsum(X,group)/sizes; Y <- as.numeric(rowsum(y,group))/sizes
Xm <- Z[as.integer(group),]; ym <- Y[as.integer(group)]
w <- lm.fit((X-Xm)[,-1],y-ym); stopifnot(w$rank==K-1)
e <- sum(w$residuals^2)/(N-G-K+1)
between <- lm.fit(Z,Y); stopifnot(between$rank==K)
u <- max(0,sum(between$residuals^2)/(G-K)-e*mean(1/sizes))
theta <- (1-sqrt(e/(e+sizes*u)))[as.integer(group)]
q <- lm.fit(X-theta*Xm,y-theta*ym); stopifnot(q$rank==K)
rscale <- sum(q$residuals^2)/(N-K)
# Respect pivot ordering for all independently fitted QR systems.
qr_inverse <- function(fit) {
  v <- chol2inv(qr.R(fit$qr)); v[fit$qr$pivot,fit$qr$pivot] <- v; v
}
VF <- e*qr_inverse(w); VR <- rscale*qr_inverse(q)[-1,-1]
# A different parameterization supplies an additional full-dummy FE check.
full <- lm.fit(cbind(X,model.matrix(~group)[,-1]),y)
stopifnot(full$rank==G+K-1,
  max(abs(full$coefficients[2:K]-w$coefficients))<1e-9,
  max(abs(sum(full$residuals^2)/(N-G-K+1)*qr_inverse(full)[2:K,2:K]-VF))<1e-9)
difference <- unname(w$coefficients-q$coefficients[-1]); D <- VF-VR
# Do not use phtest's absolute quadratic or a generalized inverse for bad D.
scale <- sqrt(pmax(diag(VF),diag(VR))); scaled <- D/outer(scale,scale)
L <- t(chol(scaled)); minimum_pivot <- min(diag(L)^2)
stopifnot(minimum_pivot>128*.Machine$double.eps)
stat <- drop(crossprod(difference,solve(D,difference))); p <- pchisq(stat,K-1,lower.tail=FALSE)
refs <- list()
emit <- function(path,value) refs[[length(refs)+1L]] <<- data.frame(path=path,value=unname(value))
values <- c(observation_count=N,group_count=G,minimum_group_size=min(sizes),maximum_group_size=max(sizes),
  singleton_groups=sum(sizes==1),fe_residual_df=N-G-K+1,re_residual_df=N-K,
  fe_residual_scale=e,re_residual_scale=rscale,minimum_scaled_cholesky_pivot=minimum_pivot,
  statistic=stat,df=K-1,p_value=p)
for(name in names(values)) emit(paste0('/',name),values[name])
for(i in seq_len(K-1)) {
  emit(sprintf('/fe_coefficients/%d',i-1),w$coefficients[i])
  emit(sprintf('/re_coefficients/%d',i-1),q$coefficients[i+1])
  emit(sprintf('/differences/%d',i-1),difference[i])
  for(j in seq_len(K-1)) for(kind in c('fe_covariance','re_covariance','covariance_difference'))
    emit(sprintf('/%s/%d/%d',kind,i-1,j-1),switch(kind,fe_covariance=VF,re_covariance=VR,covariance_difference=D)[i,j])
}
d$re_xb <- drop(X%*%q$coefficients); d$re_residual <- y-d$re_xb
precise <- function(x,path) {
  x[] <- lapply(x,function(v)if(is.numeric(v))ifelse(is.na(v),NA_character_,sprintf('%.17g',v))else v)
  write.csv(x,path,row.names=FALSE,quote=FALSE,na='')
}
dir.create(a[2]); precise(source,file.path(a[2],'source.csv'))
precise(d,file.path(a[2],'predictions.csv')); precise(do.call(rbind,refs),file.path(a[2],'reference.csv'))
cat(R.version.string,';',N,'rows;',G,'groups;',length(refs),'quantities; statistic',stat,'; p',p,'\n')
