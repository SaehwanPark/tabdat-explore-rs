# Deterministic balanced, unbalanced/singleton and negative-score QR controls.
a <- commandArgs(trailingOnly=TRUE)
if(length(a)!=2 || file.exists(a[1])) stop('usage: household_bp_lm_synthetic.R new-output-directory reference-script')
dir.create(a[1],recursive=TRUE)
for(case in c('balanced','unbalanced','negative')) {
  sizes <- if(case=='unbalanced') c(1,3:17) else rep(8,16)
  group <- rep(seq_along(sizes),sizes); n <- length(group); t <- seq_len(n)
  d <- data.frame(source_row=t,lnexp12m=0,age=sin(t*.73),sex=cos(t*.39),
    hhsize=sin(t*.17),farm=cos(t*.91),educ=sin(t*.53),lnhhexp=cos(t*.27),commune=group)
  # Short exact dyadic tokens avoid decimal-to-binary parser differences.
  d[3:8] <- lapply(d[3:8],function(v)round(v*16)/16)
  keep <- if(case=='unbalanced') t!=4 else rep(TRUE,n)
  X <- cbind(1,d$lnhhexp,d$age,d$sex,d$hhsize,d$farm,d$educ)
  mu <- drop(X%*%c(2,.3,-.2,.1,.4,-.3,.2))
  if(case=='negative') {
    # Project against X AND commune dummies: residual group sums are zero.
    D <- model.matrix(~factor(group[keep]))
    noise <- qr.resid(qr(cbind(X[keep,],D[,-1])),cos(t[keep]*1.13))
    d$lnexp12m[keep] <- mu[keep]+noise
  } else {
    d$lnexp12m <- mu+.8*cos(group*1.1)+.2*sin(t*1.7)
  }
  d$lnexp12m <- round(d$lnexp12m*1024)/1024
  d$lnexp12m[!keep] <- NA_real_
  out <- file.path(a[1],case); dir.create(out)
  d[c('source_row','commune')] <- lapply(d[c('source_row','commune')],function(v)sprintf('%.0f',v))
  d[2:8] <- lapply(d[2:8],function(v) ifelse(is.na(v),NA_character_,sprintf('%.10f',v)))
  input <- file.path(out,'input.csv'); write.csv(d,input,row.names=FALSE,quote=FALSE,na='')
  status <- system2(file.path(R.home('bin'),'Rscript'),c(shQuote(a[2]),shQuote(input),
    shQuote(file.path(out,'reference')),'--csv'))
  stopifnot(status==0)
}
