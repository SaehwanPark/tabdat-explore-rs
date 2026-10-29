# Deterministic tiny, redistributable source; run only in a new directory.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==1L, !file.exists(args[1]))
dir.create(args[1])
rows <- lapply(1:40, function(i) {
  logr <- vapply(0:9,function(t) 0.1*i+0.07*t+0.005*((i*t)%%11),numeric(1))
  pat <- vapply(0:9,function(t) (i*7+t*11+i*t*3)%%13,numeric(1))
  c(i+1000,if(i==3) '.' else as.character(10+i%%5),i%%2,
    sprintf('%.5f',1+i/100),sum(pat),sprintf('%.5f',logr),pat)
})
writeLines(vapply(rows,paste,collapse=' ',FUN.VALUE=character(1)),file.path(args[1],'source.asc'))
