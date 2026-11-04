# Independent deterministic panel and plm classical Hausman reference.
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==1,!file.exists(args[1]))
id <- rep(1:40,each=5); year <- rep(1:5,40); i <- seq_along(id)
x <- 0.7*sin(id*0.31)+cos(i*0.27)+0.1*year
y <- 1+0.8*x+0.6*sin(id*0.31)+0.3*sin(i*0.73)+0.15*cos(i*0.17)
d <- data.frame(id,year,y,x)
fe <- plm::plm(y~x,data=d,index=c('id','year'),model='within')
re <- plm::plm(y~x,data=d,index=c('id','year'),model='random',random.method='swar')
bf <- unname(coef(fe)[1]); br <- unname(coef(re)['x']); vf <- unname(vcov(fe)[1,1]); vr <- unname(vcov(re)['x','x'])
delta <- bf-br; variance <- vf-vr; stopifnot(variance>0)
h <- delta^2/variance; p <- pchisq(h,1,lower.tail=FALSE)
stopifnot(abs(unname(plm::phtest(fe,re)$statistic)-h)<1e-10)
values <- c(fe_coefficient=bf,re_coefficient=br,fe_variance=vf,re_variance=vr,difference=delta,
 variance_difference=variance,statistic=h,df=1,p_value=p,observation_count=nrow(d),group_count=40,group_size=5)
dir.create(args[1]); write.csv(d,file.path(args[1],'data.csv'),row.names=FALSE,quote=FALSE)
write.csv(data.frame(field=names(values),value=unname(values)),file.path(args[1],'reference.csv'),row.names=FALSE,quote=FALSE)
print(values)
