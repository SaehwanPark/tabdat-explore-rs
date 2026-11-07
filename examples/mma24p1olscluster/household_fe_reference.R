# Independent full-dummy QR for commune FE, including normalized intercept.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (!(length(a) %in% c(2,3)) || (length(a)==3 && !a[3]%in%c('--csv','--dta')))
  stop('usage: household_fe_reference.R source output-directory [--csv|--dta]')
if (file.exists(a[2])) stop('reference directory already exists')
if (length(a)==3 && a[3]=='--csv') {
  source <- read.csv(a[1])
} else if (length(a)==3 && a[3]=='--dta') {
  raw <- foreign::read.dta(a[1],convert.factors=FALSE)
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
      age=num(raw[,2]),sex=num(raw[,3]),hhsize=num(raw[,4]),farm=unname(farms[raw[,5]]),
      educ=unname(education[raw[,6]]),lnhhexp=num(raw[,7]),commune=num(raw[,8]))
  stopifnot(!anyNA(source[,-2]),sum(is.na(source$lnexp12m))==993)
}
d <- source[!is.na(source$lnexp12m),]
ids <- sort(unique(d$commune)); g <- length(ids); n <- nrow(d)
ns <- as.numeric(table(factor(d$commune,levels=ids)))
stopifnot(g>=2,!anyDuplicated(d$source_row))
X <- cbind(1,d$lnhhexp,d$age,d$sex,d$hhsize,d$farm,d$educ)
# Parameterize the final dummy so that sum_i n_i*u_i = 0 exactly.
D <- sapply(seq_len(g-1),function(i) as.numeric(d$commune==ids[i])-
              ns[i]/ns[g]*as.numeric(d$commune==ids[g]))
Xfull <- cbind(X,D); fit <- lm.fit(Xfull,d$lnexp12m)
stopifnot(fit$rank==ncol(Xfull)); df <- n-fit$rank; rss <- sum(fit$residuals^2)
Vfull <- matrix(0,ncol(Xfull),ncol(Xfull))
Vfull[fit$qr$pivot,fit$qr$pivot] <- rss/df*chol2inv(qr.R(fit$qr))
b <- unname(fit$coefficients[1:7]); V <- Vfull[1:7,1:7]
pooled <- lm.fit(X,d$lnexp12m); stopifnot(pooled$rank==ncol(X))
rss0 <- sum(pooled$residuals^2)
within_total <- sum((d$lnexp12m-ave(d$lnexp12m,d$commune,FUN=mean))^2)
Funit <- ((rss0-rss)/(g-1))/(rss/df)
refs <- list()
emit <- function(kind,path,value) {
  refs[[length(refs)+1L]] <<- data.frame(kind=kind,path=path,value=unname(value))
}
for (v in names(c(common_intercept=1,observation_count=1,group_count=1,df_residual=1,
                  residual_sum_of_squares=1,within_total_sum_of_squares=1,within_r_squared=1,
                  conditional_r_squared=1,root_mse=1,minimum_group_size=1,maximum_group_size=1,singleton_groups=1))) {
  value <- switch(v,common_intercept=b[1],observation_count=n,group_count=g,df_residual=df,
    residual_sum_of_squares=rss,within_total_sum_of_squares=within_total,
    within_r_squared=1-rss/within_total,conditional_r_squared=1-rss/sum((d$lnexp12m-mean(d$lnexp12m))^2),
    root_mse=sqrt(rss/df),minimum_group_size=min(ns),maximum_group_size=max(ns),singleton_groups=sum(ns==1))
  emit('fit',paste0('/',v),value)
}
for (i in 1:7) {
  emit('fit',sprintf('/normalized_level/parameters/%d',i-1),b[i])
  for (j in 1:7) emit('fit',sprintf('/normalized_level/covariance_matrix/%d/%d',i-1,j-1),V[i,j])
  if (i>1) {
    se <- sqrt(V[i,i]); t <- b[i]/se
    for (name in c('value','standard_error','statistic','p_value'))
      emit('fit',sprintf('/coefficients/%d/%s',i-2,name),
        switch(name,value=b[i],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df)))
  }
}
for (name in c('f_statistic','p_value','df_num','df_denom'))
  emit('fit',paste0('/unit_effects_f_test/',name),switch(name,f_statistic=Funit,
       p_value=pf(Funit,g-1,df,lower.tail=FALSE),df_num=g-1,df_denom=df))
for (i in 1:2) {
  se <- sqrt(V[i,i]); t <- b[i]/se
  values <- c(estimate=b[i],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df),
              ci_lower=b[i]-qt(.975,df)*se,ci_upper=b[i]+qt(.975,df)*se,df_residual=df)
  for (name in names(values)) emit(if(i==1) 'common' else 'slope',paste0('/',name),values[[name]])
}
w <- drop(t(b[1:2])%*%solve(V[1:2,1:2],b[1:2]))/2
emit('joint','/statistic',w);emit('joint','/p_value',pf(w,2,df,lower.tail=FALSE))
emit('joint','/df',2);emit('joint','/df_residual',df)
dir.create(a[2])
write.csv(do.call(rbind,refs),file.path(a[2],'reference.csv'),row.names=FALSE,quote=FALSE)
# Preserve original binary values for exact source audits, including numeric DTA codes.
precise_source <- as.data.frame(lapply(source,function(v)ifelse(is.na(v),NA_character_,sprintf('%.17g',v))))
write.csv(precise_source,file.path(a[2],'source.csv'),row.names=FALSE,quote=FALSE,na='')
d$fe_xb <- drop(X%*%b); d$fe_xbu <- fit$fitted.values
d$fe_u <- fit$fitted.values-d$fe_xb; d$fe_residual <- fit$residuals
write.csv(d,file.path(a[2],'predictions.csv'),row.names=FALSE,quote=FALSE,na='')
cat(R.version.string,';',n,'observations;',g,'communes;',length(refs),'quantities\n')
