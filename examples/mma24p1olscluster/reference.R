# Independent QR/sandwich reference for the publisher's wrapped ASCII source.
args <- commandArgs(trailingOnly=TRUE)
if (!(length(args) %in% c(2,3)) || (length(args)==3 && args[3]!='--csv'))
  stop('usage: reference.R source output-directory [--csv]')
if (length(args)==3) {
  d <- read.csv(args[1])
} else {
  tokens <- scan(args[1], what=character(), quote='"', quiet=TRUE)
  if (length(tokens) != 5999*8) stop('unexpected source size')
  a <- matrix(tokens, ncol=8, byrow=TRUE)
  farm <- c('non farm'=0, 'farm'=1)
  educ <- c('Never'=0, 'Nhatre'=1, '<cap I'=2, 'Cap I'=3, 'Cap II'=4,
            'Cap III'=5, 'Nghe SC'=6, 'THCN'=7, 'DHDC'=8, 'DHCD'=9,
            'Thac sy'=10, 'PTS'=11, 'TS'=12)
  number <- function(x) as.numeric(replace(x, x=='.', NA))
  d <- data.frame(source_row=seq_len(nrow(a)), lnexp12m=number(a[,1]),
                  age=number(a[,2]), sex=number(a[,3]), hhsize=number(a[,4]),
                  farm=unname(farm[a[,5]]), educ=unname(educ[a[,6]]),
                  lnhhexp=number(a[,7]), commune=number(a[,8]))
  if (anyNA(d[, -2]) || sum(is.na(d$lnexp12m)) != 993) stop('bad source')
}
source <- d
d <- d[!is.na(d$lnexp12m), ]
X <- cbind(intercept=1, lnhhexp=d$lnhhexp, age=d$age, sex=d$sex,
           hhsize=d$hhsize, farm=d$farm, educ=d$educ)
y <- d$lnexp12m
n <- nrow(X); k <- ncol(X); g <- length(unique(d$commune))
qrX <- qr(X)
if (qrX$rank != k || g < 2) stop('invalid design')
b <- qr.coef(qrX, y)
fitted <- as.vector(X %*% b); e <- y-fitted
bread <- chol2inv(qr.R(qrX))
scores <- X*e
cluster_scores <- rowsum(scores, d$commune, reorder=TRUE)
V <- list(nonrobust=sum(e^2)/(n-k)*bread,
          robust=n/(n-k)*bread %*% crossprod(scores) %*% bread,
          cluster=g/(g-1)*(n-1)/(n-k)*bread %*% crossprod(cluster_scores) %*% bread)
rows <- list(); j <- 1
add <- function(model, path, value) {
  rows[[j]] <<- data.frame(model=model, path=path, value=value); j <<- j+1
}
for (model in names(V)) {
  se <- sqrt(diag(V[[model]])); t <- b/se
  df <- if (model=='cluster') g-1 else n-k
  for (i in seq_len(k)) {
    for (field in c('value','standard_error','statistic','p_value')) {
      val <- switch(field, value=b[i], standard_error=se[i], statistic=t[i],
                    p_value=2*pt(-abs(t[i]),df))
      add(model, sprintf('/coefficients/%d/%s', i-1, field), val)
    }
    for (h in seq_len(k))
      add(model, sprintf('/covariance_matrix/%d/%d', i-1,h-1), V[[model]][i,h])
  }
  add(model, '/r_squared', 1-sum(e^2)/sum((y-mean(y))^2))
  add(model, '/adjusted_r_squared', 1-(sum(e^2)/(n-k))/(sum((y-mean(y))^2)/(n-1)))
  add(model, '/root_mse', sqrt(sum(e^2)/(n-k)))
}
options(digits=17)
dir.create(args[2], showWarnings=FALSE, recursive=TRUE)
write.csv(do.call(rbind, rows), file.path(args[2],'fits.csv'), row.names=FALSE, na='')
se <- sqrt(V$cluster[2,2]); t <- b[2]/se; df <- g-1; p <- 2*pt(-abs(t), df)
post <- data.frame(result=c(rep('lincom',6),rep('test',2)),
                   path=c('/estimate','/standard_error','/statistic','/p_value',
                          '/ci_lower','/ci_upper','/statistic','/p_value'),
                   value=c(b[2],se,t,p,b[2]-qt(.975,df)*se,b[2]+qt(.975,df)*se,t^2,p))
write.csv(post, file.path(args[2],'post.csv'), row.names=FALSE, na='')
write.csv(source, file.path(args[2],'source.csv'), row.names=FALSE, na='')
d$household_fitted <- fitted; d$household_residual <- e
write.csv(d, file.path(args[2],'predictions.csv'), row.names=FALSE, na='')
