# Development-only explanation probe, not a Stata implementation or parity claim.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L,!file.exists(args[2]))
tokens <- scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)==346L*25L)
src <- matrix(tokens,ncol=25L,byrow=TRUE)
v <- suppressWarnings(matrix(as.numeric(src),ncol=25L))
x0 <- as.vector(t(v[,11:15,drop=FALSE])); y <- as.vector(t(v[,21:25,drop=FALSE])); id <- rep(1:346,each=5)
float32 <- function(x) {
  c <- rawConnection(raw(),'r+b'); on.exit(close(c))
  writeBin(as.double(x),c,size=4); seek(c,0); readBin(c,'numeric',length(x),size=4)
}
refit_nine <- function(x) {
  beta <- unname(coef(glm(y~x,family=poisson())))
  for(iter in 1:9) {
    mu <- exp(beta[1]+beta[2]*x); e <- (y-mu)/sqrt(mu)
    E <- matrix(e,nrow=346,byrow=TRUE)
    rho <- sum(rowSums(E)^2-rowSums(E*E))/(4*sum(E*E))
    stopifnot(rho>-.25,rho<1)
    j <- cbind(sqrt(mu),sqrt(mu)*x); unit_j <- rowsum(j,id); unit_e <- rowsum(e,id)
    k <- 1/(1-rho); c <- rho/(1+4*rho)
    score <- k*(crossprod(j,e)-c*crossprod(unit_j,unit_e))
    bread <- k*(crossprod(j)-c*crossprod(unit_j))
    beta <- beta+drop(solve(bread,score))
  }
  beta
}
double <- refit_nine(x0); single <- refit_nine(float32(x0))
# Author's log prints a=2.067515 and b=.5595302 after nine iterations.
stopifnot(abs(single[1]-2.067515)<.0000005,abs(single[2]-.5595302)<.00000005)
write.csv(data.frame(input=c('float64','float32'),iteration=9,intercept=c(double[1],single[1]),slope=c(double[2],single[2])),args[2],row.names=FALSE,quote=FALSE)
