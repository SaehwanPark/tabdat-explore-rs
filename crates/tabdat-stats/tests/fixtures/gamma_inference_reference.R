# Independent numerical regression fixture; development-only base R.
# Run from repository root:
# Rscript crates/tabdat-stats/tests/fixtures/gamma_inference_reference.R
options(digits=17)
rows <- list()
add <- function(kind, x, df1=0, df2=0, expected) {
  rows[[length(rows)+1L]] <<- data.frame(kind, x, df1, df2, expected)
}
for (x in c(1e-8, .01, .1, .49, .5, .75, 1, 1.5, 2, 3, 5, 10, 50, 100, 1000))
  add('lgamma', x, expected=lgamma(x))
for (df in c(.5, 1, 2, 3, 10, 30, 100)) {
  for (x in c(.1, 1, 3, 10, 100))
    add('t_tail', x, df, expected=2*pt(-abs(x), df))
  for (p in c(.025, .5, .975, .999))
    add('t_quantile', p, df, expected=qt(p, df))
}
for (df1 in c(1, 2, 5, 20)) for (df2 in c(1, 3, 10, 100))
  for (x in c(.1, 1, 10, 1e8))
    add('f_tail', x, df1, df2, expected=pf(x, df1, df2, lower.tail=FALSE))
for (x in c(-30, -10, -8, -3, -1, -.1, 0, .1, 1, 3, 8))
  add('normal', x, expected=pnorm(x))
write.csv(do.call(rbind, rows),
  'crates/tabdat-stats/tests/fixtures/gamma_inference_reference.csv', row.names=FALSE, quote=FALSE)
cat(R.version.string, '\n')
