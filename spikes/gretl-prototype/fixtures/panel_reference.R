library(plm)
d <- read.csv('/work/spikes/gretl-prototype/fixtures/panel.csv')
for (family in c('fe','re')) {
  fit <- plm(y ~ x1+x2,data=d,index=c('id','time'),model=if(family=='fe') 'within' else 'random',random.method='swar')
  stopifnot(nobs(fit)==192)
  values <- c(coef(fit),sqrt(diag(vcov(fit))),as.vector(t(vcov(fit))))
  cat('R,',family,',',paste(sprintf('%.17g',values),collapse=','),'\n',sep='')
}
