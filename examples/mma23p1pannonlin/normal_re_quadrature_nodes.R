# Generate the in-script physicists' Gauss-Hermite nodes/weights; development-only.
args<-commandArgs(trailingOnly=TRUE);stopifnot(length(args)%in%c(1L,3L),!file.exists(args[1]))
levels<-if(length(args)==1L)c(80L,96L) else as.integer(args[2:3])
stopifnot(!anyNA(levels),levels[1]>=2L,levels[2]>levels[1],levels[2]<=128L)
out<-file(args[1],open='wx');on.exit(close(out))
for(n in levels){
  J<-matrix(0,n,n);for(k in 1:(n-1))J[k,k+1]<-J[k+1,k]<-sqrt(k/2)
  z<-sort(eigen(J,symmetric=TRUE,only.values=TRUE)$values)
  p0<-rep(1,n);p1<-sqrt(2)*z
  for(k in 2:(n-1)){p2<-sqrt(2/k)*z*p1-sqrt((k-1)/k)*p0;p0<-p1;p1<-p2}
  # Eigenvectors can lose tiny first components; the orthogonal-polynomial
  # recurrence recovers positive tail weights without that cancellation.
  w<-sqrt(pi)/(n*p1^2);stopifnot(all(w>0),abs(sum(w)-sqrt(pi))<1e-12)
  for(i in seq_len(n))writeLines(sprintf(' (%d,%d,%.17g::DOUBLE,%.17g::DOUBLE)%s',n,i,z[i],w[i],if(n==levels[2]&&i==n)'' else ','),out)
}
