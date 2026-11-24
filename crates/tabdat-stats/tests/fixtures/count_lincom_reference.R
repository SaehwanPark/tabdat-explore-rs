# Derive independent normal-reference linear combinations from validated fits.
# Run poisson_offset_reference.R and nb2_offset_reference.R first.
options(digits=17)
poisson <- read.csv('crates/tabdat-stats/tests/fixtures/poisson_offset_reference.csv')
nb2 <- read.csv('crates/tabdat-stats/tests/fixtures/nb2_offset_reference.csv')
rows <- list()
value <- function(reference,case,quantity,i,j=NULL) {
  keep <- reference$case==case & reference$quantity==quantity & reference$i==i
  if (!is.null(j)) keep <- keep & reference$j==j
  selected <- reference$value[keep]
  stopifnot(length(selected)==1L)
  selected
}
reference <- function(family,fit_reference,covariance) {
  case <- paste0('multiple_',covariance)
  beta <- vapply(0:2,function(i)
    value(fit_reference,case,'coefficient_value',i),numeric(1))
  V <- outer(0:2,0:2,Vectorize(function(i,j)
    value(fit_reference,case,'covariance',i,j)))
  combinations <- list(
    slope=list(weights=c(0,1,0),offset=0),
    contrast=list(weights=c(0,1,-1),offset=2),
    sum=list(weights=c(0,1,1),offset=-0.25)
  )
  for (name in names(combinations)) {
    combination <- combinations[[name]]
    estimate <- combination$offset+sum(combination$weights*beta)
    standard_error <- sqrt(drop(t(combination$weights)%*%V%*%combination$weights))
    statistic <- estimate/standard_error
    rows[[length(rows)+1L]] <<- data.frame(
      family,covariance,combination=name,estimate,standard_error,statistic,
      p_value=2*pnorm(-abs(statistic)),
      ci_lower=estimate-qnorm(0.975)*standard_error,
      ci_upper=estimate+qnorm(0.975)*standard_error,
      ci_level=95
    )
  }
}
for (covariance in c('model','robust','cluster')) {
  reference('poisson',poisson,covariance)
  reference('nb2',nb2,covariance)
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
