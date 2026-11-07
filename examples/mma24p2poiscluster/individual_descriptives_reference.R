# Independent variable-wise source samples; wholly/partly missing records are not dropped.
options(digits=17)
a <- commandArgs(trailingOnly=TRUE)
if (!(length(a)%in%c(2,3)) || (length(a)==3 && !a[3]%in%c('--csv','--dta')))
  stop('usage: individual_descriptives_reference.R source fresh-directory [--csv|--dta]')
if (file.exists(a[2])) stop('reference directory already exists')
variables <- c('pharvis','lnhhexp','age','sex','married','educ','illness','injury','illdays','actdays','insurance','commune')
keys <- c('source_row',variables)
if (length(a)==3 && a[3]=='--csv') {
  source <- read.csv(a[1],check.names=FALSE)
} else if (length(a)==3 && a[3]=='--dta') {
  raw <- foreign::read.dta(a[1],convert.factors=FALSE)
  stopifnot(nrow(raw)==27766,all(is.na(raw[27734,])))
  source <- data.frame(source_row=seq_len(nrow(raw)),pharvis=raw$PHARVIS,lnhhexp=raw$lnhhinc,
    age=raw$AGE,sex=raw$SEX,married=raw$MARRIED,educ=raw$COMPED98,illness=raw$ILLDUM,
    injury=raw$INJDUM,illdays=raw$ILLDAYS,actdays=raw$ACTDAYS,insurance=raw$HLTHINS,commune=raw$commune)
} else {
  tokens <- scan(a[1],what=character(),quiet=TRUE)
  stopifnot(length(tokens)==27766*12)
  raw <- matrix(tokens,ncol=12,byrow=TRUE)
  values <- matrix(as.numeric(replace(raw,raw=='.',NA)),ncol=12)
  stopifnot(identical(which(rowSums(is.na(values))>0),27734L),all(is.na(values[27734,])))
  source <- data.frame(source_row=seq_len(nrow(values)),values)
  names(source) <- keys
}
stopifnot(identical(names(source),keys),nrow(source)>=1,nrow(source)<=40000,!anyDuplicated(source$source_row))
stopifnot(all(is.finite(source$source_row)),all(source$source_row>0),all(source$source_row<=2^53-1),all(source$source_row==floor(source$source_row)))
for (key in c('pharvis','commune')) {
  x <- source[[key]]
  stopifnot(all(is.na(x)|(is.finite(x)&x>=if(key=='pharvis')0 else 1)),all(is.na(x)|(x<=2^53-1&x==floor(x))))
}
for (key in variables) stopifnot(all(is.na(source[[key]])|is.finite(source[[key]])))
p <- source[order(source$source_row),]
p[variables] <- lapply(p[variables],as.numeric)
summary <- do.call(rbind,lapply(variables,function(name){
  x <- p[[name]];x <- x[is.finite(x)]
  data.frame(variable=name,count=length(x),mean=if(length(x))mean(x)else NA_real_,
    std_dev=if(length(x)>1)sd(x)else NA_real_,minimum=if(length(x))min(x)else NA_real_,
    maximum=if(length(x))max(x)else NA_real_)
}))
members <- do.call(rbind,lapply(variables,function(name){
  ids <- p$source_row[is.finite(p[[name]])]
  data.frame(variable=rep(name,length(ids)),source_row=ids)
}))
precise <- function(d,path){
  d[] <- lapply(d,function(v)if(is.numeric(v))ifelse(is.na(v),NA_character_,sprintf('%.17g',v))else v)
  write.csv(d,path,row.names=FALSE,quote=FALSE,na='')
}
dir.create(a[2])
for (name in c('source','summary','members')) precise(get(name),file.path(a[2],paste0(name,'.csv')))
precise(p,file.path(a[2],'prepared_source.csv'))
writeLines(R.version.string,file.path(a[2],'version.txt'))
cat(R.version.string,';',nrow(source),'source rows;',nrow(summary)*5,'variable-wise summary fields\n')
