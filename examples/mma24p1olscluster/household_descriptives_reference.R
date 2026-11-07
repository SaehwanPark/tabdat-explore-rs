# Independent variable-wise source samples; no listwise/model-cohort selection.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (!(length(a)%in%c(2,3)) || (length(a)==3 && !a[3]%in%c('--csv','--dta')))
  stop('usage: household_descriptives_reference.R source fresh-directory [--csv|--dta]')
if (file.exists(a[2])) stop('reference directory already exists')
keys <- c('source_row','lnexp12m','age','sex','hhsize','farm','educ','lnhhexp','commune')
if (length(a)==3 && a[3]=='--csv') {
  source <- read.csv(a[1],check.names=FALSE)
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
  education <- c('Never'=0,'Nhatre'=1,'<cap I'=2,'Cap I'=3,'Cap II'=4,'Cap III'=5,'Nghe SC'=6,'THCN'=7,'DHDC'=8,'DHCD'=9,'Thac sy'=10,'PTS'=11,'TS'=12)
  num <- function(x)as.numeric(replace(x,x=='.',NA))
  source <- data.frame(source_row=seq_len(nrow(raw)),lnexp12m=num(raw[,1]),
    age=num(raw[,2]),sex=num(raw[,3]),hhsize=num(raw[,4]),farm=unname(farms[raw[,5]]),
    educ=unname(education[raw[,6]]),lnhhexp=num(raw[,7]),commune=num(raw[,8]))
}
stopifnot(identical(names(source),keys),nrow(source)>=1,nrow(source)<=40000,!anyDuplicated(source$source_row))
for (key in c('source_row','commune')) stopifnot(all(is.finite(source[[key]])),all(source[[key]]>0),all(source[[key]]<=2^53-1),all(source[[key]]==floor(source[[key]])))
for (key in keys[-c(1,9)]) stopifnot(all(is.na(source[[key]])|is.finite(source[[key]])))
p <- source[order(source$source_row),]
p[keys[-c(1,9)]] <- lapply(p[keys[-c(1,9)]],as.numeric)
p$hhexp <- exp(p$lnhhexp)
stopifnot(all(is.na(p$hhexp)|(is.finite(p$hhexp)&p$hhexp>0)))
variables <- c('lnexp12m','age','sex','hhsize','farm','educ','hhexp','lnhhexp','commune')
summarize <- function(d)do.call(rbind,lapply(variables,function(name){
  x <- d[[name]];x <- x[is.finite(x)]
  data.frame(variable=name,count=length(x),mean=if(length(x))mean(x)else NA_real_,
    std_dev=if(length(x)>1)sd(x)else NA_real_,minimum=if(length(x))min(x)else NA_real_,
    maximum=if(length(x))max(x)else NA_real_)
}))
summary <- summarize(p)
members <- do.call(rbind,lapply(variables,function(name){
  ids <- p$source_row[is.finite(p[[name]])]
  data.frame(variable=rep(name,length(ids)),source_row=ids)
}))
# Source gen defaults to float; retain a separate diagnostic, not a native oracle swap.
legacy <- p
legacy$hhexp <- readBin(writeBin(p$hhexp,raw(),size=4,endian='little'),numeric(),n=nrow(p),size=4,endian='little')
precise <- function(d,path){
  d[] <- lapply(d,function(v)if(is.numeric(v))ifelse(is.na(v),NA_character_,sprintf('%.17g',v))else v)
  write.csv(d,path,row.names=FALSE,quote=FALSE,na='')
}
dir.create(a[2])
for (name in c('source','summary','members')) precise(get(name),file.path(a[2],paste0(name,'.csv')))
precise(p,file.path(a[2],'prepared_source.csv'))
precise(summarize(legacy),file.path(a[2],'legacy_float_summary.csv'))
writeLines(R.version.string,file.path(a[2],'version.txt'))
cat(R.version.string,';',nrow(source),'source rows;',nrow(summary)*5,'variable-wise summary fields\n')
