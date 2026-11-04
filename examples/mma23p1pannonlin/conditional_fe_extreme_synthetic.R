# Independent synthetic slope controls with large |z|; no original data required.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==3L, args[3] %in% c('positive','negative','near_positive','near_negative'), !file.exists(args[2]))
dir.create(args[2])
tokens <- scan(args[1],what=character(),quiet=TRUE)
stopifnot(length(tokens)>0L,length(tokens)%%25L==0L)
src <- matrix(tokens,ncol=25L,byrow=TRUE)
for (i in seq_len(nrow(src))) {
  near <- startsWith(args[3],'near_')
  src[i,11:15] <- as.character((0:4)*if(near) .4 else 1)
  counts <- if(near) c(0,0,0,1,50) else c(1,1,2,4,8)
  src[i,21:25] <- as.character(if(endsWith(args[3],'positive')) counts else rev(counts))
}
writeLines(apply(src,1,paste,collapse=' '),file.path(args[2],'source.asc'))
