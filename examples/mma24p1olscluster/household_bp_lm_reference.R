# Independent pooled QR and plm unbalanced individual-effect BP LM.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (!(length(a) %in% c(2,3)) || (length(a)==3 && !a[3] %in% c('--csv','--dta')))
  stop('usage: household_bp_lm_reference.R source output-directory [--csv|--dta]')
if (file.exists(a[2])) stop('reference directory already exists')
mode <- if(length(a)==3) a[3] else '--ascii'
if (mode=='--csv') {
  source <- read.csv(a[1])
} else if (mode=='--dta') {
  # Factor expansion changes the seven-parameter source design and its LM.
  d <- foreign::read.dta(a[1],convert.factors=FALSE)
  source <- data.frame(source_row=seq_len(nrow(d)),lnexp12m=d$lhhex12m,
    age=d$age,sex=d$sex,hhsize=d$hhsize,farm=d$farm,educ=d$comped98,
    lnhhexp=d$lhhexp1,commune=d$commune)
} else {
  tokens <- scan(a[1],what=character(),quote='"',quiet=TRUE)
  stopifnot(length(tokens)==5999*8)
  raw <- matrix(tokens,ncol=8,byrow=TRUE)
  farms <- c('non farm'=0,'farm'=1)
  education <- c('Never'=0,'Nhatre'=1,'<cap I'=2,'Cap I'=3,'Cap II'=4,
    'Cap III'=5,'Nghe SC'=6,'THCN'=7,'DHDC'=8,'DHCD'=9,'Thac sy'=10,'PTS'=11,'TS'=12)
  num <- function(x) as.numeric(replace(x,x=='.',NA))
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=num(raw[,1]),
    age=num(raw[,2]),sex=num(raw[,3]),hhsize=num(raw[,4]),farm=unname(farms[raw[,5]]),
    educ=unname(education[raw[,6]]),lnhhexp=num(raw[,7]),commune=num(raw[,8]))
}
stopifnot(!anyNA(source[,-2]),!anyDuplicated(source$source_row))
d <- source[!is.na(source$lnexp12m),]; d <- d[order(d$source_row),]
X <- cbind(1,d$lnhhexp,d$age,d$sex,d$hhsize,d$farm,d$educ)
fit <- lm.fit(X,d$lnexp12m); stopifnot(fit$rank==7)
e <- fit$residuals; n <- nrow(d); sizes <- as.numeric(table(d$commune)); g <- length(sizes)
rss <- sum(e^2); q <- sum(tapply(e,d$commune,sum)^2); m <- sum(sizes^2)
z <- n*(q/rss-1)/sqrt(2*(m-n)); lm <- z^2; p <- pchisq(lm,1,lower.tail=FALSE)
pfit <- plm::plm(lnexp12m~lnhhexp+age+sex+hhsize+farm+educ,data=d,
  index=c('commune','source_row'),model='pooling')
bp <- plm::plmtest(pfit,effect='individual',type='bp')
honda <- plm::plmtest(pfit,effect='individual',type='honda')
stopifnot(abs(as.numeric(bp$statistic)-lm)<1e-9,abs(as.numeric(honda$statistic)-z)<1e-9,
  abs(bp$p.value-p)<=max(abs(p)*1e-8,1e-300))
diag <- data.frame(observations=n,communes=g,minimum_group_size=min(sizes),
  maximum_group_size=max(sizes),singleton_groups=sum(sizes==1),sum_group_size_squared=m,
  group_residual_sum_squares=q,residual_sum_of_squares=rss,
  centered_outcome_sum_squares=sum((d$lnexp12m-mean(d$lnexp12m))^2),
  signed_score=z,lm_statistic=lm,df=1,p_value=p)
d$bp_xb <- fit$fitted.values; d$bp_residual <- e
V <- rss/(n-7)*chol2inv(qr.R(fit$qr))
V[fit$qr$pivot,fit$qr$pivot] <- V
fitrefs <- data.frame(path=sprintf('/coefficients/%d/value',0:6),value=unname(fit$coefficients))
for(i in 1:7) for(j in 1:7) fitrefs <- rbind(fitrefs,
  data.frame(path=sprintf('/covariance_matrix/%d/%d',i-1,j-1),value=V[i,j]))
precise <- function(x,path) {
  x[] <- lapply(x,function(v) if(is.numeric(v)) ifelse(is.na(v),NA_character_,sprintf('%.17g',v)) else v)
  write.csv(x,path,row.names=FALSE,na='',quote=FALSE)
}
dir.create(a[2])
precise(source,file.path(a[2],'source.csv'))
precise(d,file.path(a[2],'predictions.csv'))
precise(diag,file.path(a[2],'diagnostic.csv'))
precise(fitrefs,file.path(a[2],'fit.csv'))
cat(R.version.string,'; plm',as.character(packageVersion('plm')),';',n,'rows;',g,
  'groups; LM',lm,'; signed score',z,'; p',p,'\n')
