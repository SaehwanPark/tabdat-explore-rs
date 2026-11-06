# Independent R QR refits with a separately implemented Python xoshiro draw oracle.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if(length(a)<2 || length(a)>3 || (length(a)==3 && !a[3]%in%c('--csv','--dta')))
  stop('usage: household_ols_bootstrap_reference.R source output-directory [--csv | --dta]')
if(file.exists(a[2])) stop('reference directory already exists')
mode <- if(length(a)==3) a[3] else 'ascii'
if(mode=='--csv') {
  source <- read.csv(a[1])
} else if(mode=='--dta') {
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
stopifnot(all(required%in%names(source)),all(vapply(source[required],is.numeric,logical(1))),
    all(is.finite(source$source_row)),all(source$source_row>=1),
    all(source$source_row==floor(source$source_row)),!anyDuplicated(source$source_row),
    all(is.finite(source$commune)),all(source$commune>=1),all(source$commune==floor(source$commune)))
selected <- apply(source[c('lnexp12m',predictors)],1,function(row)all(is.finite(row)))
d <- source[selected,];d <- d[order(d$source_row),]
f <- lnexp12m~lnhhexp+age+sex+hhsize+farm+educ
X <- model.matrix(f,d);y <- d$lnexp12m;N <- length(y);K <- ncol(X);ids <- unique(d$commune)
G <- length(ids);B <- 500L;seed <- 10001L
stopifnot(G>=2,N>K,qr(X)$rank==K)
members <- lapply(ids,function(id)which(d$commune==id));sizes <- lengths(members)
script <- sub('^--file=','',commandArgs()[grepl('^--file=',commandArgs())])
repo <- normalizePath(file.path(dirname(script),'../..'))
index_file <- tempfile(fileext='.csv')
status <- system2('python3',c(shQuote(file.path(repo,'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py')),
    '--draws',seed,G,B,shQuote(index_file)))
stopifnot(status==0)
indices <- read.csv(index_file,colClasses='character')
stopifnot(nrow(indices)==B,identical(as.integer(indices$replication),seq_len(B)))
rep_counts <- integer(B)
bs <- t(vapply(seq_len(B),function(r) {
  chosen <- as.integer(strsplit(indices$indices[r],':',fixed=TRUE)[[1]])+1L
  stopifnot(length(chosen)==G,all(chosen>=1),all(chosen<=G))
  index <- unlist(members[chosen],use.names=FALSE);rep_counts[r] <<- length(index)
  point <- lm.fit(X[index,,drop=FALSE],y[index]);stopifnot(point$rank==K,all(is.finite(point$coefficients)))
  unname(point$coefficients)
},numeric(K)))
base <- lm.fit(X,y);stopifnot(base$rank==K)
beta <- unname(base$coefficients);V <- cov(bs);means <- colMeans(bs);refs <- list()
emit <- function(path,value) {stopifnot(length(value)==1,is.finite(value));refs[[length(refs)+1L]] <<- data.frame(path=path,value=unname(value))}
for(i in seq_len(K)) {
  ci <- quantile(bs[,i],c(.025,.975),type=7,names=FALSE)
  vals <- c(estimate=beta[i],bootstrap_mean=means[i],bias=means[i]-beta[i],
      standard_error=sqrt(V[i,i]),ci_lower=ci[1],ci_upper=ci[2])
  for(name in names(vals)) emit(paste0('/coefficients/',i-1,'/',name),vals[[name]])
  for(j in seq_len(K)) emit(paste0('/covariance_matrix/',i-1,'/',j-1),V[i,j])
}
emit('/observation_count',N);emit('/replications',B);emit('/cluster_count',G)
d$pooled_fitted <- drop(X%*%beta);d$pooled_residual <- y-d$pooled_fitted
stopifnot(length(refs)==94,!anyDuplicated(vapply(refs,function(x)x$path,'')))
if(!dir.create(a[2],recursive=TRUE)) stop('cannot create reference directory')
write.csv(source,file.path(a[2],'input.csv'),row.names=FALSE,na='')
write.csv(d,file.path(a[2],'predictions.csv'),row.names=FALSE,na='')
write.csv(do.call(rbind,refs),file.path(a[2],'reference.csv'),row.names=FALSE)
write.csv(data.frame(replication=seq_len(B),observations=rep_counts),file.path(a[2],'replicate_counts.csv'),row.names=FALSE)
write.csv(data.frame(commune=ids,observation_count=sizes),file.path(a[2],'groups.csv'),row.names=FALSE)
stopifnot(file.copy(index_file,file.path(a[2],'draws.csv')));unlink(index_file)
cat(R.version.string,'N',N,'G',G,'B',B,'seed',seed,'bootstrap SE',sqrt(diag(V)),
    'replicate N range',range(rep_counts),'\n')
