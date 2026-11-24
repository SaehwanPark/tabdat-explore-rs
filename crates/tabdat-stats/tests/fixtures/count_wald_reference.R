# Derive independent Wald references from the validated Poisson/NB2 offset fits.
# Run the poisson_offset_reference.R and nb2_offset_reference.R generators first.
options(digits=17)
poisson <- read.csv('crates/tabdat-stats/tests/fixtures/poisson_offset_reference.csv')
nb2 <- read.csv('crates/tabdat-stats/tests/fixtures/nb2_offset_reference.csv')
rows <- list()
value <- function(reference, case, quantity, i, j=NULL) {
  keep <- reference$case==case & reference$quantity==quantity & reference$i==i
  if (!is.null(j)) keep <- keep & reference$j==j
  selected <- reference$value[keep]
  stopifnot(length(selected)==1L)
  selected
}
reference <- function(family, fit_reference, covariance) {
  case <- paste0('multiple_',covariance)
  beta <- vapply(0:2,function(i)value(fit_reference,case,'coefficient_value',i),numeric(1))
  V <- outer(0:2,0:2,Vectorize(function(i,j)
    value(fit_reference,case,'covariance',i,j)))
  restrictions <- list(
    one=list(R=matrix(c(0,1,0),nrow=1L),r=0),
    contrast=list(R=matrix(c(0,1,-1),nrow=1L),r=0.25),
    joint=list(R=rbind(c(0,1,0),c(0,0,1)),r=c(0,0))
  )
  for (name in names(restrictions)) {
    restriction <- restrictions[[name]]
    difference <- drop(restriction$R%*%beta)-restriction$r
    tested_covariance <- restriction$R%*%V%*%t(restriction$R)
    statistic <- drop(t(difference)%*%solve(tested_covariance,difference))
    df <- nrow(restriction$R)
    rows[[length(rows)+1L]] <<- data.frame(
      family,covariance,restriction=name,df,statistic,
      p_value=pchisq(statistic,df,lower.tail=FALSE)
    )
  }
}
for (covariance in c('model','robust','cluster')) {
  reference('poisson',poisson,covariance)
  reference('nb2',nb2,covariance)
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
