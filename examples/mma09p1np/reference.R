# Independent direct R kernel sums, not SQL/FFT reference ordinates.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args) %in% c(2L,3L), !dir.exists(args[2]))
options(digits=17)
raw <- read.csv(args[1], check.names=FALSE)
fields <- c('intnum','persnum','age','educatn','earnings','hours','sex','kids','married')
stopifnot(identical(names(raw),c('source_row',fields)))
if (length(args)==3L) {
  original <- read.fwf(args[3], widths=c(4,3,2,2,6,4,1,2,1),
    col.names=fields, colClasses='numeric', na.strings=c('','.'), strip.white=TRUE)
  original <- data.frame(source_row=seq_len(nrow(original)),original)
  stopifnot(identical(is.na(raw),is.na(original)),
    all(as.matrix(raw)==as.matrix(original),na.rm=TRUE))
  raw <- original
}
x <- raw[!is.na(raw$educatn) & raw$educatn %in% 1:97 & raw$earnings>0 & raw$age==36,]
x$capped_earnings <- pmin(x$earnings,100000)
x$hwage <- x$capped_earnings/x$hours
x$lnhwage <- log(x$hwage)
x <- x[order(x$source_row),]
stopifnot(nrow(x)>1,all(is.finite(x$lnhwage)),all(x$hours>0))
v <- x$lnhwage; n <- length(v)
quantiles <- quantile(v,c(.25,.75),names=FALSE,type=7)
scale <- min(sd(v),diff(quantiles)/1.349)
h <- 1.3643*1.7188*scale/n^.2
stopifnot(is.finite(h),h>0)
bandwidth <- data.frame(n=n,sample_mean=mean(v),sample_sd=sd(v),q25=quantiles[1],
  q75=quantiles[2],scale=scale,bandwidth=h,min_x=min(v),max_x=max(v))
grid <- seq(min(v)-2*h,max(v)+2*h,length.out=512)
# Vectorized observation-by-evaluation distances; SQL instead groups a cross join.
kde <- function(at,width) {
  u <- outer(v,at,'-')/width
  colMeans(.75*pmax(1-u*u,0))/width
}
density <- data.frame(grid_index=0:511,log_wage_grid=grid,
  density_half=kde(grid,h/2),density_plugin=kde(grid,h),density_double=kde(grid,2*h))
# Historical default generated-float storage is a separate author-output audit,
# never the native oracle. as.single() alone does NOT round R values to float32.
f32 <- function(z) readBin(writeBin(z,raw(),size=4,endian='little'),
  what='double',n=length(z),size=4,endian='little')
legacy <- f32(log(f32(x$capped_earnings/x$hours)))
legacy_scale <- min(sd(legacy),diff(quantile(legacy,c(.25,.75),names=FALSE,type=7))/1.349)
legacy_stats <- data.frame(n=n,sample_mean=mean(legacy),sample_sd=sd(legacy),
  scale=legacy_scale,bandwidth=1.3643*1.7188*legacy_scale/n^.2,
  gaussian_bandwidth=1.3643*.7764*legacy_scale/n^.2,
  biweight_bandwidth=1.3643*2.0362*legacy_scale/n^.2,
  rectangle_bandwidth=.5*1.3643*1.3510*legacy_scale/n^.2)
# stats::density uses SD-normalized kernels, so conventional Epan halfwidth h
# means bw=h/sqrt(5). It uses binned FFT/interpolation, not exact direct sums.
# K is1.5-Lipschitz. Bound data binning + internal-grid interpolation +
# interpolation to our grid. R pads its FFT range by ext*bw on each side,
# so internal spacing is NOT just diff(fit$x). Use conservative full-cell errors.
# This secondary discretization bound never replaces primary direct-R tolerances.
certificates <- do.call(rbind,lapply(c(.5,1,2),function(multiplier) {
  width <- multiplier*h
  do.call(rbind,lapply(c(2^16,2^18),function(resolution) {
    fit <- density(v,kernel='epanechnikov',bw=width/sqrt(5),n=resolution,
      from=min(grid),to=max(grid),old.coords=FALSE)
    predicted <- approx(fit$x,fit$y,xout=grid)$y
    error <- max(abs(predicted-kde(grid,width)))
    internal_dx <- (diff(range(grid))+8*width/sqrt(5))/(resolution-1)
    bound <- 1.5*(2*internal_dx+max(diff(fit$x)))/width^2
    stopifnot(is.finite(error),error<=bound+1e-11)
    data.frame(multiplier=multiplier,resolution=resolution,max_absolute_error=error,
      interpolation_bound=bound)
  }))
}))
# Integrating every quadratic piece by Simpson is exact, not a grid mass renormalization.
normalization <- sapply(c(.5,1,2),function(m) {
  width <- m*h; knots <- sort(unique(c(v-width,v+width)))
  left <- head(knots,-1);right <- tail(knots,-1)
  sum((right-left)*(kde(left,width)+4*kde((left+right)/2,width)+kde(right,width))/6)
})
stopifnot(max(abs(normalization-1))<1e-12)
dir.create(args[2])
write_precise <- function(frame,name) {
  text <- as.data.frame(lapply(frame,function(z)ifelse(is.na(z),'',sprintf('%.17g',z))),
    check.names=FALSE)
  write.table(text,file.path(args[2],name),sep=',',row.names=FALSE,quote=FALSE,na='')
}
for (entry in list(list(raw,'source.csv'),list(x,'sample.csv'),
  list(bandwidth,'bandwidth.csv'),list(density,'density.csv'),
  list(legacy_stats,'legacy_statistics.csv'),list(certificates,'fft_certificate.csv'),
  list(data.frame(multiplier=c(.5,1,2),mass=normalization),'normalization.csv'))) {
  write_precise(entry[[1]],entry[[2]])
}
writeLines(c(R.version.string,paste('stats',packageVersion('stats'))),
  file.path(args[2],'versions.txt'))
cat(n,'selected rows;1536 direct-R ordinates; three exact-piece masses; six FFT certificates\n')
