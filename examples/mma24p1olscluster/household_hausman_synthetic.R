# Small exact-dyadic balanced/unbalanced sources for the separate-scale test.
a <- commandArgs(trailingOnly=TRUE)
if(length(a)!=2 || file.exists(a[1])) stop('usage: household_hausman_synthetic.R new-output-directory reference-script')
dir.create(a[1],recursive=TRUE)
for(case in c('balanced','unbalanced')) {
  sizes <- if(case=='balanced') rep(6,32) else c(1,rep(4:9,length.out=31))
  group <- rep(seq_along(sizes),sizes); n <- length(group); i <- seq_len(n)
  X <- sapply(1:6,function(j)round((sin(group*(.17+j*.23))+.2*sin(i*(.31+j*.19)))*16)/16)
  y <- 2+drop(X%*%c(.3,-.2,.1,.4,-.3,.2))+.25*cos(group*.83)+sin(i*1.4)+.7*cos(i*.63)
  d <- data.frame(source_row=i,lnexp12m=round(y*1024)/1024,age=X[,2],sex=X[,3],
    hhsize=X[,4],farm=X[,5],educ=X[,6],lnhhexp=X[,1],commune=group)
  if(case=='unbalanced') d$lnexp12m[4] <- NA_real_
  out <- file.path(a[1],case); dir.create(out)
  d[c('source_row','commune')] <- lapply(d[c('source_row','commune')],function(v)sprintf('%.0f',v))
  d[2:8] <- lapply(d[2:8],function(v)ifelse(is.na(v),NA_character_,sprintf('%.10f',v)))
  input <- file.path(out,'input.csv'); write.csv(d,input,row.names=FALSE,quote=FALSE,na='')
  status <- system2(file.path(R.home('bin'),'Rscript'),c(shQuote(a[2]),shQuote(input),
    shQuote(file.path(out,'reference')),'--csv'))
  stopifnot(status==0)
}
