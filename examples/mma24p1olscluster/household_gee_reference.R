# Optional independent reference, not a TabDat runtime dependency.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (length(a)<2 || length(a)>3 ||
    (length(a)==3 && !a[3]%in%c('--csv','--dta')))
  stop('usage: household_gee_reference.R source output-directory [--csv | --dta]')
if (file.exists(a[2])) stop('reference directory already exists')
stopifnot(packageVersion('geeM')==package_version('0.10.1'))
mode <- if(length(a)==3) a[3] else 'ascii'
if (mode=='--csv') {
  source <- read.csv(a[1])
} else if (mode=='--dta') {
  raw <- foreign::read.dta(a[1],convert.factors=FALSE)
  stopifnot(nrow(raw)==5999,sum(is.na(raw$lhhex12m))==993)
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=raw$lhhex12m,
      age=raw$age,sex=raw$sex,hhsize=raw$hhsize,farm=raw$farm,
      educ=raw$comped98,lnhhexp=raw$lhhexp1,commune=raw$commune)
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
      age=num(raw[,2]),sex=num(raw[,3]),hhsize=num(raw[,4]),
      farm=unname(farms[raw[,5]]),educ=unname(education[raw[,6]]),
      lnhhexp=num(raw[,7]),commune=num(raw[,8]))
  stopifnot(!anyNA(source[,-2]),sum(is.na(source$lnexp12m))==993)
}
predictors <- c('lnhhexp','age','sex','hhsize','farm','educ')
required <- c('source_row','lnexp12m',predictors,'commune')
stopifnot(all(required%in%names(source)),!anyNA(source$source_row),
    !anyDuplicated(source$source_row),
    all(vapply(source[required],is.numeric,logical(1))))
selected <- apply(source[c('lnexp12m',predictors,'commune')],1,
    function(row) all(is.finite(row)))
d <- source[selected,required]
# Package clustering requires contiguous IDs; output rows retain original order.
d <- d[order(d$commune,d$source_row),]
f <- lnexp12m~lnhhexp+age+sex+hhsize+farm+educ
X <- model.matrix(f,d);y <- d$lnexp12m;N <- length(y);K <- ncol(X)
blocks <- split(seq_len(N),factor(d$commune,levels=unique(d$commune)))
G <- length(blocks);sizes <- lengths(blocks);pair_count <- sum(sizes*(sizes-1))
stopifnot(G>K,N>G,pair_count>0,qr(X)$rank==K)
evaluate <- function(b) {
  residual <- drop(y-X%*%b);rss <- sum(residual^2);phi <- rss/N
  rho <- sum(vapply(blocks,function(i)
      sum(residual[i])^2-sum(residual[i]^2),0.0))/(phi*pair_count)
  stopifnot(is.finite(phi),phi>0,is.finite(rho),rho<1,
      1+(max(sizes)-1)*rho>0)
  H <- matrix(0,K,K);xy <- numeric(K);scores <- matrix(0,G,K)
  for(j in seq_along(blocks)) {
    i <- blocks[[j]];z <- X[i,,drop=FALSE]
    # Dense inverses deliberately differ from the planned Rust whitening/QR path.
    V <- phi*((1-rho)*diag(length(i))+rho*matrix(1,length(i),length(i)))
    IV <- solve(V)
    H <- H+crossprod(z,IV%*%z);xy <- xy+drop(crossprod(z,IV%*%y[i]))
    scores[j,] <- drop(crossprod(z,IV%*%residual[i]))
  }
  W <- solve(H);influence <- scores%*%W
  list(phi=phi,rho=rho,working=W,sandwich=crossprod(influence),
      scores=scores,influence=influence,next_beta=drop(solve(H,xy)))
}
start <- unname(coef(lm(f,d)));b <- start;converged <- FALSE
for(it in 1:100) {
  e <- evaluate(b);next_b <- e$next_beta
  if(max(abs(next_b-b)/(1+abs(b)))<=1e-12) {
    b <- next_b;converged <- TRUE;break
  }
  b <- next_b
}
stopifnot(converged,it<100)
e <- evaluate(b)
stopifnot(all(abs(e$working%*%colSums(e$scores))<=1e-10*(1+abs(b))))
# useP=FALSE removes dispersion/correlation DF corrections, not sandwich factors.
m <- geeM::geem(f,id=commune,data=d,family=gaussian,corstr='exchangeable',
    init.beta=start,useP=FALSE,tol=1e-12,maxit=100)
stopifnot(m$converged,m$niter<100,max(abs(m$beta-b))<1e-9,
    abs(m$phi-e$phi)<1e-9,abs(m$alpha-e$rho)<1e-9,
    max(abs(as.matrix(m$naiv.var)-e$working))<1e-9,
    max(abs(as.matrix(m$var)-e$sandwich))<1e-9)
terms <- colnames(X);terms[1] <- '_cons'
refs <- list()
emit <- function(path,value) {
  stopifnot(length(value)==1,is.finite(value))
  refs[[length(refs)+1L]] <<- data.frame(path=path,value=unname(value))
}
summary <- c(source_count=nrow(source),observation_count=N,group_count=G,
    minimum_group_size=min(sizes),maximum_group_size=max(sizes),
    singleton_group_count=sum(sizes==1),parameter_count=K,
    ordered_pair_count=pair_count,phi=e$phi,rho=e$rho)
for(name in names(summary)) emit(name,summary[[name]])
for(kind in c('working','sandwich')) {
  covariance <- e[[kind]];se <- unname(sqrt(diag(covariance)));z <- unname(b/se)
  for(j in seq_len(K)) {
    prefix <- paste(kind,terms[j],sep='.')
    fields <- c(estimate=unname(b[j]),standard_error=se[j],statistic=z[j],
        p_value=2*pnorm(-abs(z[j])),ci_lower=unname(b[j])-qnorm(.975)*se[j],
        ci_upper=unname(b[j])+qnorm(.975)*se[j])
    for(name in names(fields)) emit(paste(prefix,name,sep='.'),fields[[name]])
    for(k in seq_len(K))
      emit(paste(kind,'covariance',terms[j],terms[k],sep='.'),covariance[j,k])
  }
  wald <- drop(t(b[-1])%*%solve(covariance[-1,-1],b[-1]))
  emit(paste(kind,'wald_statistic',sep='.'),wald)
  emit(paste(kind,'wald_df',sep='.'),K-1)
  emit(paste(kind,'wald_p_value',sep='.'),pchisq(wald,K-1,lower.tail=FALSE))
}
source_prediction <- rep(NA_real_,nrow(source))
finite_x <- apply(source[predictors],1,function(row)all(is.finite(row)))
Xp <- model.matrix(~lnhhexp+age+sex+hhsize+farm+educ,source[finite_x,])
source_prediction[finite_x] <- drop(Xp%*%b)
predictions <- data.frame(source_row=source$source_row,selected=as.integer(selected),
    mean=source_prediction,xb=source_prediction,
    residual=source$lnexp12m-source_prediction)
predictions$residual[!is.finite(predictions$residual)] <- NA_real_
unit <- data.frame(commune=vapply(blocks,function(i)d$commune[i[1]],0.0),
    observation_count=sizes)
for(j in seq_len(K)) {
  unit[[paste0('score_',terms[j])]] <- e$scores[,j]
  unit[[paste0('influence_',terms[j])]] <- e$influence[,j]
}
stopifnot(!anyDuplicated(vapply(refs,function(x)x$path,'')))
if(!dir.create(a[2],recursive=TRUE)) stop('cannot create reference directory')
write.csv(do.call(rbind,refs),file.path(a[2],'reference.csv'),row.names=FALSE)
write.csv(source,file.path(a[2],'input.csv'),row.names=FALSE,na='')
write.csv(predictions,file.path(a[2],'predictions.csv'),row.names=FALSE,na='')
write.csv(unit,file.path(a[2],'units.csv'),row.names=FALSE)
cat('Validated',length(refs),'quantities;',N,'cases;',G,'groups; dense/geeM iterations',
    it,m$niter,'; phi',e$phi,'; rho',e$rho,'\n')
