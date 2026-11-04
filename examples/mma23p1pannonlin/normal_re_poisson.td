# Bounded shared normal random-intercept Poisson.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id!=floor(id) OR id<1 OR id>346 OR year!=floor(year) OR year<1975 OR year>1979 OR pat<0 OR pat!=floor(pat) OR pat>1000 OR ABS(logr)>20)>0 THEN error('invalid normal RE panel data') ELSE COUNT(*) END AS observations,COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*)=0 OR MIN(n)!=5 OR MAX(n)!=5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id,COUNT(*) AS n FROM active GROUP BY id)"""
poisson pat logr
predict pooled_xb, xb
sql """
WITH RECURSIVE
source AS (SELECT * FROM active),
firm AS (SELECT id,COUNT(*) AS n,SUM(pat ORDER BY year) AS y_total,SUM(pat*logr ORDER BY year) AS yx,SUM(ABS(pat*logr) ORDER BY year) AS yx_scale,SUM(lgamma(pat+1) ORDER BY year) AS factorial FROM source GROUP BY id),
start_regression AS (SELECT REGR_INTERCEPT(pooled_xb,logr) AS a,REGR_SLOPE(pooled_xb,logr) AS b FROM source),
start AS (
 SELECT CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b)
   OR COUNT(*) FILTER(WHERE pooled_xb IS NULL OR NOT isfinite(pooled_xb))>0
   OR MAX(ABS(pooled_xb-(a+b*logr)))>1e-12*(1+MAX(ABS(pooled_xb)))
   THEN error('normal RE pooled start cannot be reconstructed') ELSE a END AS a,b FROM source CROSS JOIN start_regression GROUP BY a,b
),
-- Physicists' Hermite nodes and weights; posterior-mode centering is per firm and iteration.
quadrature(level,node,z,weight) AS (VALUES
 (80,1,-11.887863560471134::DOUBLE,2.9557746033001513e-62::DOUBLE),
 (80,2,-11.281694164989267::DOUBLE,2.8621845857332944e-56::DOUBLE),
 (80,3,-10.780796472469138::DOUBLE,1.5671564937760379e-51::DOUBLE),
 (80,4,-10.334491039538372::DOUBLE,1.7642057043387966e-47::DOUBLE),
 (80,5,-9.9234351145692532::DOUBLE,6.7924330161519454e-44::DOUBLE),
 (80,6,-9.5376679163020768::DOUBLE,1.1692076998190594e-40::DOUBLE),
 (80,7,-9.1712174787264775::DOUBLE,1.0595146928279446e-37::DOUBLE),
 (80,8,-8.8201501286610675::DOUBLE,5.6371927794743458e-35::DOUBLE),
 (80,9,-8.4817021873215594::DOUBLE,1.902990610032897e-32::DOUBLE),
 (80,10,-8.1538381574634826::DOUBLE,4.3178173430184087e-30::DOUBLE),
 (80,11,-7.8350036783293078::DOUBLE,6.8834570436297408e-28::DOUBLE),
 (80,12,-7.5239772908068678::DOUBLE,7.9851871981731911e-26::DOUBLE),
 (80,13,-7.2197764697589211::DOUBLE,6.9341198195296031e-24::DOUBLE),
 (80,14,-6.9215953729258315::DOUBLE,4.6134710745522466e-22::DOUBLE),
 (80,15,-6.6287620808576158::DOUBLE,2.3978402756169088e-20::DOUBLE),
 (80,16,-6.3407083213299789::DOUBLE,9.8965289896216683e-19::DOUBLE),
 (80,17,-6.0569474734510287::DOUBLE,3.2891620143645261e-17::DOUBLE),
 (80,18,-5.7770582280615717::DOUBLE,8.9094610406658026e-16::DOUBLE),
 (80,19,-5.5006722123151484::DOUBLE,1.9875208346988428e-14::DOUBLE),
 (80,20,-5.2274644551006695::DOUBLE,3.6848498201596406e-13::DOUBLE),
 (80,21,-4.9571459285134285::DOUBLE,5.7232797185020024e-12::DOUBLE),
 (80,22,-4.689457632952811::DOUBLE,7.499719914658618e-11::DOUBLE),
 (80,23,-4.4241658477651864::DOUBLE,8.3430559260863988e-10::DOUBLE),
 (80,24,-4.1610582741258391::DOUBLE,7.9229232891499377e-09::DOUBLE),
 (80,25,-3.8999408693873892::DOUBLE,6.4544629501196453e-08::DOUBLE),
 (80,26,-3.6406352232272101::DOUBLE,4.5305355658612248e-07::DOUBLE),
 (80,27,-3.382976362512399::DOUBLE,2.750724627401341e-06::DOUBLE),
 (80,28,-3.1268108983731309::DOUBLE,1.4496444191129472e-05::DOUBLE),
 (80,29,-2.8719954485287316::DOUBLE,6.6517412418154464e-05::DOUBLE),
 (80,30,-2.6183952824717851::DOUBLE,0.00026647788669334824::DOUBLE),
 (80,31,-2.365883148074226::DOUBLE,0.00093431768696116855::DOUBLE),
 (80,32,-2.1143382465070073::DOUBLE,0.0028731975330897903::DOUBLE),
 (80,33,-1.8636453287449288::DOUBLE,0.0077640394053996187::DOUBLE),
 (80,34,-1.6136938918510602::DOUBLE,0.018465751154427987::DOUBLE),
 (80,35,-1.3643774570540064::DOUBLE,0.038708599914838111::DOUBLE),
 (80,36,-1.115592914603349::DOUBLE,0.071600747691843408::DOUBLE),
 (80,37,-0.86723992270397521::DOUBLE,0.1169807338042993::DOUBLE),
 (80,38,-0.61922034962756733::DOUBLE,0.16893796927590171::DOUBLE),
 (80,39,-0.37143774948304442::DOUBLE,0.21577494627433022::DOUBLE),
 (80,40,-0.1237968631731322::DOUBLE,0.24383585380721162::DOUBLE),
 (80,41,0.12379686317313153::DOUBLE,0.24383585380721179::DOUBLE),
 (80,42,0.37143774948304387::DOUBLE,0.21577494627433055::DOUBLE),
 (80,43,0.61922034962756667::DOUBLE,0.16893796927590232::DOUBLE),
 (80,44,0.86723992270397454::DOUBLE,0.11698073380429951::DOUBLE),
 (80,45,1.1155929146033481::DOUBLE,0.071600747691843714::DOUBLE),
 (80,46,1.3643774570540073::DOUBLE,0.038708599914837868::DOUBLE),
 (80,47,1.6136938918510599::DOUBLE,0.018465751154427973::DOUBLE),
 (80,48,1.8636453287449286::DOUBLE,0.00776403940539963::DOUBLE),
 (80,49,2.11433824650701::DOUBLE,0.0028731975330897257::DOUBLE),
 (80,50,2.365883148074226::DOUBLE,0.00093431768696116855::DOUBLE),
 (80,51,2.6183952824717842::DOUBLE,0.000266477886693351::DOUBLE),
 (80,52,2.8719954485287356::DOUBLE,6.6517412418151252e-05::DOUBLE),
 (80,53,3.1268108983731278::DOUBLE,1.4496444191130036e-05::DOUBLE),
 (80,54,3.3829763625124043::DOUBLE,2.7507246274011479e-06::DOUBLE),
 (80,55,3.6406352232272052::DOUBLE,4.5305355658615414e-07::DOUBLE),
 (80,56,3.8999408693873909::DOUBLE,6.4544629501194574e-08::DOUBLE),
 (80,57,4.1610582741258373::DOUBLE,7.9229232891501478e-09::DOUBLE),
 (80,58,4.4241658477651837::DOUBLE,8.3430559260867845e-10::DOUBLE),
 (80,59,4.6894576329528066::DOUBLE,7.499719914659263e-11::DOUBLE),
 (80,60,4.9571459285134294::DOUBLE,5.7232797185018909e-12::DOUBLE),
 (80,61,5.227464455100665::DOUBLE,3.6848498201599859e-13::DOUBLE),
 (80,62,5.5006722123151564::DOUBLE,1.9875208346984922e-14::DOUBLE),
 (80,63,5.7770582280615717::DOUBLE,8.9094610406658026e-16::DOUBLE),
 (80,64,6.0569474734510278::DOUBLE,3.2891620143645982e-17::DOUBLE),
 (80,65,6.3407083213299833::DOUBLE,9.8965289896205763e-19::DOUBLE),
 (80,66,6.6287620808576184::DOUBLE,2.3978402756167451e-20::DOUBLE),
 (80,67,6.9215953729258217::DOUBLE,4.6134710745535011e-22::DOUBLE),
 (80,68,7.2197764697589184::DOUBLE,6.9341198195301086e-24::DOUBLE),
 (80,69,7.5239772908068794::DOUBLE,7.9851871981704303e-26::DOUBLE),
 (80,70,7.8350036783293122::DOUBLE,6.8834570436287588e-28::DOUBLE),
 (80,71,8.1538381574634684::DOUBLE,4.3178173430204504e-30::DOUBLE),
 (80,72,8.4817021873215701::DOUBLE,1.902990610032189e-32::DOUBLE),
 (80,73,8.8201501286610728::DOUBLE,5.6371927794732767e-35::DOUBLE),
 (80,74,9.1712174787264882::DOUBLE,1.0595146928275377e-37::DOUBLE),
 (80,75,9.5376679163020892::DOUBLE,1.1692076998185011e-40::DOUBLE),
 (80,76,9.9234351145692443::DOUBLE,6.7924330161542962e-44::DOUBLE),
 (80,77,10.33449103953834::DOUBLE,1.7642057043411295e-47::DOUBLE),
 (80,78,10.780796472469159::DOUBLE,1.5671564937745916e-51::DOUBLE),
 (80,79,11.281694164989274::DOUBLE,2.8621845857323613e-56::DOUBLE),
 (80,80,11.88786356047112::DOUBLE,2.9557746033021791e-62::DOUBLE),
 (96,1,-13.116130021662869::DOUBLE,1.3153371477016118e-75::DOUBLE),
 (96,2,-12.529230746683501::DOUBLE,3.4808413877253695e-69::DOUBLE),
 (96,3,-12.044806741134952::DOUBLE,4.4687024216749156e-64::DOUBLE),
 (96,4,-11.61362082884291::DOUBLE,1.0933824737519661e-59::DOUBLE),
 (96,5,-11.216875194202148::DOUBLE,8.7337328359246419e-56::DOUBLE),
 (96,6,-10.844888073853578::DOUBLE,3.0222939315033001e-52::DOUBLE),
 (96,7,-10.491854539276547::DOUBLE,5.3832225069683194e-49::DOUBLE),
 (96,8,-10.153951274713688::DOUBLE,5.5370648197825605e-46::DOUBLE),
 (96,9,-9.8284927461917224::DOUBLE,3.5689385617796308e-43::DOUBLE),
 (96,10,-9.5135017694124322::DOUBLE,1.5317989647107622e-40::DOUBLE),
 (96,11,-9.2074693533717564::DOUBLE,4.5874016658589187e-38::DOUBLE),
 (96,12,-8.9092105814595826::DOUBLE,9.9468000592267021e-36::DOUBLE),
 (96,13,-8.6177732442147406::DOUBLE,1.6088732079387881e-33::DOUBLE),
 (96,14,-8.3323773071700504::DOUBLE,1.9895468942701312e-31::DOUBLE),
 (96,15,-8.0523733307197141::DOUBLE,1.9199707067947304e-29::DOUBLE),
 (96,16,-7.7772130310612377::DOUBLE,1.4712415549742001e-27::DOUBLE),
 (96,17,-7.5064278944277651::DOUBLE,9.0859656470464907e-26::DOUBLE),
 (96,18,-7.2396132940083442::DOUBLE,4.5806010985510788e-24::DOUBLE),
 (96,19,-6.9764164642851689::DOUBLE,1.9062601886882077e-22::DOUBLE),
 (96,20,-6.7165272404965197::DOUBLE,6.6129511764809277e-21::DOUBLE),
 (96,21,-6.4596708195551287::DOUBLE,1.9288793268367916e-19::DOUBLE),
 (96,22,-6.2056020247183143::DOUBLE,4.7668311711861169e-18::DOUBLE),
 (96,23,-5.9541007064117277::DOUBLE,1.0049051491574473e-16::DOUBLE),
 (96,24,-5.7049680135255416::DOUBLE,1.8181691304929729e-15::DOUBLE),
 (96,25,-5.4580233400707803::DOUBLE,2.8387673951697004e-14::DOUBLE),
 (96,26,-5.2131018018189641::DOUBLE,3.8436817248763409e-13::DOUBLE),
 (96,27,-4.9700521331667655::DOUBLE,4.5333033745845267e-12::DOUBLE),
 (96,28,-4.7287349203529212::DOUBLE,4.6760103859859019e-11::DOUBLE),
 (96,29,-4.4890211062169971::DOUBLE,4.2335937835108545e-10::DOUBLE),
 (96,30,-4.2507907159031859::DOUBLE,3.3755847311277311e-09::DOUBLE),
 (96,31,-4.0139317636282712::DOUBLE,2.3773662147573399e-08::DOUBLE),
 (96,32,-3.7783393087968604::DOUBLE,1.4829694050050591e-07::DOUBLE),
 (96,33,-3.5439146360273077::DOUBLE,8.2135445478332807e-07::DOUBLE),
 (96,34,-3.3105645385243081::DOUBLE,4.0482331787770929e-06::DOUBLE),
 (96,35,-3.0782006880468176::DOUBLE,1.7791805918220556e-05::DOUBLE),
 (96,36,-2.8467390777247812::DOUBLE,6.9854324445416658e-05::DOUBLE),
 (96,37,-2.616099526362389::DOUBLE,0.00024541809273052634::DOUBLE),
 (96,38,-2.3862052347697777::DOUBLE,0.00077269582168166542::DOUBLE),
 (96,39,-2.1569823861931301::DOUBLE,0.0021831319760966879::DOUBLE),
 (96,40,-1.9283597841455031::DOUBLE,0.0055416312894389502::DOUBLE),
 (96,41,-1.7002685219383493::DOUBLE,0.012651375112800281::DOUBLE),
 (96,42,-1.4726416790227106::DOUBLE,0.026000340271244256::DOUBLE),
 (96,43,-1.2454140399065081::DOUBLE,0.04813990673108999::DOUBLE),
 (96,44,-1.0185218319476799::DOUBLE,0.080353950080642042::DOUBLE),
 (96,45,-0.79190247875401676::DOUBLE,0.1209831168625153::DOUBLE),
 (96,46,-0.56549436626674887::DOUBLE,0.164379630598497::DOUBLE),
 (96,47,-0.33923661887888523::DOUBLE,0.20161301347923408::DOUBLE),
 (96,48,-0.11306888315145175::DOUBLE,0.22327002349753491::DOUBLE),
 (96,49,0.1130688831514519::DOUBLE,0.22327002349753486::DOUBLE),
 (96,50,0.33923661887888562::DOUBLE,0.20161301347923441::DOUBLE),
 (96,51,0.56549436626674854::DOUBLE,0.16437963059849672::DOUBLE),
 (96,52,0.79190247875401543::DOUBLE,0.12098311686251571::DOUBLE),
 (96,53,1.0185218319476794::DOUBLE,0.080353950080642236::DOUBLE),
 (96,54,1.245414039906509::DOUBLE,0.048139906731089747::DOUBLE),
 (96,55,1.4726416790227097::DOUBLE,0.026000340271244437::DOUBLE),
 (96,56,1.7002685219383498::DOUBLE,0.012651375112800237::DOUBLE),
 (96,57,1.928359784145502::DOUBLE,0.0055416312894389867::DOUBLE),
 (96,58,2.1569823861931279::DOUBLE,0.00218313197609673::DOUBLE),
 (96,59,2.3862052347697777::DOUBLE,0.00077269582168166542::DOUBLE),
 (96,60,2.6160995263623907::DOUBLE,0.00024541809273052103::DOUBLE),
 (96,61,2.8467390777247803::DOUBLE,6.9854324445417309e-05::DOUBLE),
 (96,62,3.0782006880468149::DOUBLE,1.7791805918221138e-05::DOUBLE),
 (96,63,3.3105645385243063::DOUBLE,4.0482331787771954e-06::DOUBLE),
 (96,64,3.5439146360273095::DOUBLE,8.21354454783307e-07::DOUBLE),
 (96,65,3.7783393087968631::DOUBLE,1.4829694050049985e-07::DOUBLE),
 (96,66,4.0139317636282668::DOUBLE,2.3773662147575076e-08::DOUBLE),
 (96,67,4.2507907159031779::DOUBLE,3.3755847311281919e-09::DOUBLE),
 (96,68,4.4890211062169909::DOUBLE,4.2335937835113358e-10::DOUBLE),
 (96,69,4.7287349203529265::DOUBLE,4.6760103859854172e-11::DOUBLE),
 (96,70,4.97005213316677::DOUBLE,4.5333033745841172e-12::DOUBLE),
 (96,71,5.2131018018189579::DOUBLE,3.8436817248768311e-13::DOUBLE),
 (96,72,5.4580233400707892::DOUBLE,2.838767395169145e-14::DOUBLE),
 (96,73,5.7049680135255318::DOUBLE,1.8181691304933724e-15::DOUBLE),
 (96,74,5.9541007064117304::DOUBLE,1.0049051491573846e-16::DOUBLE),
 (96,75,6.2056020247183215::DOUBLE,4.7668311711852664e-18::DOUBLE),
 (96,76,6.4596708195551251::DOUBLE,1.9288793268369638e-19::DOUBLE),
 (96,77,6.7165272404965135::DOUBLE,6.6129511764820163e-21::DOUBLE),
 (96,78,6.9764164642851672::DOUBLE,1.9062601886883102e-22::DOUBLE),
 (96,79,7.2396132940083575::DOUBLE,4.5806010985493031e-24::DOUBLE),
 (96,80,7.5064278944277687::DOUBLE,9.0859656470455609e-26::DOUBLE),
 (96,81,7.7772130310612386::DOUBLE,1.4712415549741699e-27::DOUBLE),
 (96,82,8.052373330719707::DOUBLE,1.9199707067951741e-29::DOUBLE),
 (96,83,8.3323773071700504::DOUBLE,1.9895468942701312e-31::DOUBLE),
 (96,84,8.617773244214753::DOUBLE,1.6088732079380956e-33::DOUBLE),
 (96,85,8.9092105814595843::DOUBLE,9.9468000592261074e-36::DOUBLE),
 (96,86,9.2074693533717742::DOUBLE,4.5874016658559307e-38::DOUBLE),
 (96,87,9.5135017694124411::DOUBLE,1.531798964710252e-40::DOUBLE),
 (96,88,9.8284927461917153::DOUBLE,3.5689385617806309e-43::DOUBLE),
 (96,89,10.153951274713698::DOUBLE,5.5370648197801592e-46::DOUBLE),
 (96,90,10.491854539276542::DOUBLE,5.3832225069694787e-49::DOUBLE),
 (96,91,10.844888073853573::DOUBLE,3.0222939315040023e-52::DOUBLE),
 (96,92,11.21687519420216::DOUBLE,8.733732835919721e-56::DOUBLE),
 (96,93,11.613620828842912::DOUBLE,1.0933824737518786e-59::DOUBLE),
 (96,94,12.044806741134916::DOUBLE,4.4687024216825492e-64::DOUBLE),
 (96,95,12.529230746683506::DOUBLE,3.4808413877244918e-69::DOUBLE),
 (96,96,13.116130021662855::DOUBLE,1.3153371477025934e-75::DOUBLE)
),
search(iter,a,b,t,g,n,old_a,old_b,old_t,u0,u1,u2,s0,s1,s2,i00,i01,i02,i11,i12,i22,ll,last_step,quad_diff) AS (
 SELECT 0::BIGINT,a,b,0.0::DOUBLE,(SELECT COUNT(*) FROM firm),(SELECT COUNT(*) FROM source),
   NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,
   NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,1e100::DOUBLE,NULL::DOUBLE FROM start
 UNION ALL
 SELECT iter+1,a+factor*da,b+factor*db,t+factor*dt,g,n,a,b,t,step.u0,step.u1,step.u2,step.s0,step.s1,step.s2,
   step.i00,step.i01,step.i02,step.i11,step.i12,step.i22,step.ll,
   GREATEST(ABS(factor*da)/(1+ABS(a)),ABS(factor*db)/(1+ABS(b)),ABS(factor*dt)/(1+ABS(t))),step.quad_diff
 FROM search CROSS JOIN LATERAL (
   WITH RECURSIVE
   observation AS (
     SELECT s.*,CASE WHEN a IS NULL OR b IS NULL OR t IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR NOT isfinite(t)
       OR ABS(a)>20 OR ABS(b)>20 OR ABS(t)>=8 OR NOT isfinite(a+b*s.logr) OR ABS(a+b*s.logr)>40
       THEN error('normal RE parameters or predictor outside numerical bounds') ELSE EXP(a+b*s.logr) END AS mu FROM source s
   ), panel AS MATERIALIZED (
     SELECT id,COUNT(*) AS panel_n,SUM(pat ORDER BY year) AS sy,SUM(pat*logr ORDER BY year) AS syx,
       SUM(ABS(pat*logr) ORDER BY year) AS syx_scale,SUM(lgamma(pat+1) ORDER BY year) AS factorial,
       SUM(mu ORDER BY year) AS m,SUM(mu*logr ORDER BY year) AS h,SUM(mu*logr*logr ORDER BY year) AS j,
       EXP(t) AS variance FROM observation GROUP BY id,t
   ), mode(id,k,lo,hi) AS (
     -- The log posterior is strictly concave; bracket and check every firm's root.
     SELECT id,0::BIGINT,-60.0::DOUBLE,60.0::DOUBLE FROM panel
     UNION ALL
     SELECT m.id,k+1,CASE WHEN p.sy-p.m*EXP((lo+hi)/2)-(lo+hi)/(2*p.variance)>0 THEN (lo+hi)/2 ELSE lo END,
       CASE WHEN p.sy-p.m*EXP((lo+hi)/2)-(lo+hi)/(2*p.variance)>0 THEN hi ELSE (lo+hi)/2 END
     FROM mode m JOIN panel p USING(id) WHERE k<55
   ), centers AS (
     SELECT p.*, (lo+hi)/2 AS center,
       CASE WHEN NOT isfinite(p.sy-p.m*EXP((lo+hi)/2)-(lo+hi)/(2*p.variance))
         OR ABS(p.sy-p.m*EXP((lo+hi)/2)-(lo+hi)/(2*p.variance))>1e-11*(1+p.sy+p.m*EXP((lo+hi)/2)+ABS((lo+hi)/(2*p.variance)))
         THEN error('normal RE posterior mode failed') ELSE SQRT(1/(1/p.variance+p.m*EXP((lo+hi)/2))) END AS scale
     FROM panel p JOIN mode USING(id) WHERE k=55
   ), raw AS (
     SELECT c.*,q.level,q.node,q.z,q.weight,
       CASE WHEN NOT isfinite(c.center+SQRT(2)*c.scale*q.z) OR ABS(c.center+SQRT(2)*c.scale*q.z)>100
         THEN error('normal RE quadrature node outside numerical bounds') ELSE c.center+SQRT(2)*c.scale*q.z END AS u
     FROM centers c CROSS JOIN quadrature q
   ), terms AS (
     SELECT *,LN(weight)+sy*u-m*EXP(u)-u*u/(2*variance)+z*z AS logterm FROM raw
   ), peaks AS (SELECT id,level,MAX(logterm) AS peak FROM terms GROUP BY id,level),
   moments AS (
     SELECT x.id,x.level,MAX(panel_n) AS panel_n,MAX(sy) AS sy,MAX(syx) AS syx,MAX(syx_scale) AS syx_scale,MAX(factorial) AS factorial,
       MAX(m) AS m,MAX(h) AS h,MAX(j) AS j,MAX(variance) AS variance,MAX(scale) AS scale,MAX(peak) AS peak,
       SUM(EXP(logterm-peak) ORDER BY node) AS den,
       SUM(EXP(logterm-peak+u) ORDER BY node) AS sex,
       SUM(EXP(logterm-peak)*u*u ORDER BY node) AS su2
     FROM terms x JOIN peaks p USING(id,level) GROUP BY x.id,x.level
   ), posterior AS (
     SELECT *,peak+LN(den)+LN(SQRT(2)*scale)-0.5*LN(2*PI()*variance) AS log_integral,
       sex/den AS ex,su2/den AS eu2 FROM moments
   ), centered AS (
     -- Center before taking posterior covariances; raw second-moment subtraction
     -- loses enough information to perturb small-tail inference on these data.
     SELECT p.id,p.level,
       SUM(EXP(x.logterm-p.peak)*POWER(EXP(x.u)-p.ex,2) ORDER BY x.node)/MAX(p.den) AS var_exp,
       SUM(EXP(x.logterm-p.peak)*POWER(x.u*x.u-p.eu2,2) ORDER BY x.node)/MAX(p.den) AS var_u2,
       SUM(EXP(x.logterm-p.peak)*(EXP(x.u)-p.ex)*(x.u*x.u-p.eu2) ORDER BY x.node)/MAX(p.den) AS cov_exp_u2
     FROM terms x JOIN posterior p USING(id,level) GROUP BY p.id,p.level
   ), checked AS (
     SELECT high.*,hc.var_exp,hc.var_u2,hc.cov_exp_u2,
       GREATEST(ABS(high.log_integral-low.log_integral)/(1+ABS(high.log_integral)),
         ABS(high.ex-low.ex)/(1+ABS(high.ex)),ABS(high.eu2-low.eu2)/(1+ABS(high.eu2)),
         ABS(hc.var_exp-lc.var_exp)/(1+ABS(hc.var_exp)),ABS(hc.var_u2-lc.var_u2)/(1+ABS(hc.var_u2)),
         ABS(hc.cov_exp_u2-lc.cov_exp_u2)/(1+ABS(hc.cov_exp_u2))) AS discrepancy
     FROM posterior high JOIN posterior low USING(id) JOIN centered hc ON hc.id=high.id AND hc.level=high.level
       JOIN centered lc ON lc.id=low.id AND lc.level=low.level WHERE high.level=96 AND low.level=80
   ), system AS (
     SELECT COUNT(*) AS groups,SUM(panel_n) AS rows,MIN(panel_n) AS min_panel_n,MAX(panel_n) AS max_panel_n,MAX(discrepancy) AS quad_diff,
       SUM(sy-m*ex ORDER BY id) AS u0,SUM(syx-h*ex ORDER BY id) AS u1,SUM((eu2/variance-1)/2 ORDER BY id) AS u2,
       SUM(ABS(sy)+ABS(m*ex) ORDER BY id) AS s0,SUM(syx_scale+ABS(h*ex) ORDER BY id) AS s1,SUM((1+ABS(eu2/variance))/2 ORDER BY id) AS s2,
       SUM(m*ex-m*m*var_exp ORDER BY id) AS i00,
       SUM(h*ex-m*h*var_exp ORDER BY id) AS i01,
       SUM(m*cov_exp_u2/(2*variance) ORDER BY id) AS i02,
       SUM(j*ex-h*h*var_exp ORDER BY id) AS i11,
       SUM(ABS(j*ex)+ABS(h*h*var_exp) ORDER BY id) AS i11_scale,
       SUM(h*cov_exp_u2/(2*variance) ORDER BY id) AS i12,
       SUM(eu2/(2*variance)-var_u2/(4*variance*variance) ORDER BY id) AS i22,
       SUM(ABS(eu2/(2*variance))+ABS(var_u2)/(4*variance*variance) ORDER BY id) AS i22_scale,
       SUM(sy*a+syx*b-factorial+log_integral ORDER BY id) AS ll FROM checked
   ), first_pivot AS (
     SELECT *,CASE WHEN groups!=search.g OR rows!=search.n OR min_panel_n!=5 OR max_panel_n!=5
       OR quad_diff IS NULL OR NOT isfinite(quad_diff) OR quad_diff>1e-9
       OR NOT isfinite(u0) OR NOT isfinite(u1) OR NOT isfinite(u2) OR NOT isfinite(s0) OR NOT isfinite(s1) OR NOT isfinite(s2)
       OR NOT isfinite(i00) OR i00<=0 OR NOT isfinite(i01) OR NOT isfinite(i02) OR NOT isfinite(i11) OR NOT isfinite(i11_scale)
       OR NOT isfinite(i12) OR NOT isfinite(i22) OR NOT isfinite(i22_scale) OR NOT isfinite(ll)
       OR (iter>0 AND ll<search.ll-1e-10*(1+ABS(search.ll)))
       THEN error('normal RE incomplete, inaccurate, nonfinite or nonascending iterate') ELSE i00 END AS p00 FROM system
   ), second_pivot AS (
     SELECT *,CASE WHEN NOT isfinite(i11-i01*i01/p00)
       OR i11-i01*i01/p00<=2.842170943040401e-14*(1+i11_scale+ABS(i01*i01/p00))
       THEN error('normal RE information is singular') ELSE i11-i01*i01/p00 END AS p11 FROM first_pivot
   ), third_pivot AS (
     SELECT *, (i12-i01*i02/p00)/p11 AS l21,
       CASE WHEN NOT isfinite(i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11)
       OR i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11<=2.842170943040401e-14*(1+i22_scale+ABS(i02*i02/p00)+ABS(POWER(i12-i01*i02/p00,2)/p11))
       THEN error('normal RE information is singular') ELSE i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11 END AS p22 FROM second_pivot
   ), steps AS (SELECT *, (u2-(i02/p00)*u0-l21*(u1-(i01/p00)*u0))/p22 AS dt FROM third_pivot),
   steps2 AS (SELECT *, (u1-(i01/p00)*u0)/p11-l21*dt AS db FROM steps)
   -- The declared first half-step avoids the original sample's nonascending
   -- full Newton step. Later full steps still must pass the ascent guard.
   SELECT *, (u0-i01*db-i02*dt)/p00 AS da,CASE WHEN iter=0 THEN 0.5 ELSE 1.0 END AS factor FROM steps2
 ) step
 WHERE iter<100 AND (iter=0 OR last_step>1e-12)
), final AS MATERIALIZED (SELECT * FROM search ORDER BY iter DESC LIMIT 1),
final_checked AS (
 SELECT *,CASE WHEN iter=0 OR last_step IS NULL OR NOT isfinite(last_step) OR last_step>1e-12
   OR ABS(u0)>1e-10*(1+s0) OR ABS(u1)>1e-10*(1+s1) OR ABS(u2)>1e-10*(1+s2)
   THEN error('normal RE failed convergence or final score check') ELSE old_a END AS intercept FROM final
), inverse_base AS (
 SELECT *,i11-i01*i01/i00 AS p11,(i12-i01*i02/i00)/(i11-i01*i01/i00) AS l21,
   i22-i02*i02/i00-POWER(i12-i01*i02/i00,2)/(i11-i01*i01/i00) AS p22 FROM final_checked
), inverse_info AS (
 SELECT *,1/p22 AS c22,-l21/p22 AS c12,1/p11+l21*l21/p22 AS c11,
   -(i02/i00-l21*i01/i00)/p22 AS c02,-(i01/i00)/p11+l21*(i02/i00-l21*i01/i00)/p22 AS c01,
   1/i00+POWER(i01/i00,2)/p11+POWER(i02/i00-l21*i01/i00,2)/p22 AS c00 FROM inverse_base
), guarded_fit AS MATERIALIZED (
 SELECT *,CASE WHEN NOT isfinite(c00) OR NOT isfinite(c01) OR NOT isfinite(c02) OR NOT isfinite(c11) OR NOT isfinite(c12) OR NOT isfinite(c22)
   OR c00<=0 OR c11<=0 OR c22<=0
   OR 2.842170943040401e-14*(ABS(c00)*s0+ABS(c01)*s1+ABS(c02)*s2)>=1e-9+1e-10*ABS(intercept)
   OR 2.842170943040401e-14*(ABS(c01)*s0+ABS(c11)*s1+ABS(c12)*s2)>=1e-9+1e-10*ABS(old_b)
   OR 2.842170943040401e-14*(ABS(c02)*s0+ABS(c12)*s1+ABS(c22)*s2)>=1e-9+1e-10*ABS(old_t)
   THEN error('normal RE covariance or coefficient is numerically unreliable') ELSE old_b END AS slope FROM inverse_info
), native AS (SELECT SUM(pat*pooled_xb-EXP(pooled_xb)-lgamma(pat+1) ORDER BY id,year) AS poisson_log_likelihood FROM source)
SELECT s.*,projection.marginal_mu,s.pat-projection.marginal_mu AS marginal_residual,
 f.intercept AS estimate_intercept,f.slope AS estimate_slope,f.old_t AS estimate_lnvariance,EXP(f.old_t/2) AS sigma,
 f.c00 AS covariance_aa,f.c01 AS covariance_ab,f.c02 AS covariance_at,f.c11 AS covariance_bb,f.c12 AS covariance_bt,f.c22 AS covariance_tt,
 f.u0 AS score_intercept,f.u1 AS score_slope,f.u2 AS score_lnvariance,f.ll AS log_likelihood,n.poisson_log_likelihood,
 CASE WHEN NOT isfinite(n.poisson_log_likelihood) OR NOT isfinite(2*(f.ll-n.poisson_log_likelihood))
   OR f.ll-n.poisson_log_likelihood<=2.842170943040401e-14*(1+ABS(f.ll)+ABS(n.poisson_log_likelihood))
   THEN error('normal RE likelihood is indistinguishable from its Poisson boundary') ELSE 2*(f.ll-n.poisson_log_likelihood) END AS lr_variance_zero,
 f.iter AS iterations,f.last_step,f.quad_diff,(SELECT COUNT(*) FROM source) AS candidate_rows,(SELECT COUNT(*) FROM firm) AS firms,
 (SELECT COUNT(*) FROM firm WHERE y_total=0) AS zero_total_firms
FROM source s CROSS JOIN guarded_fit f CROSS JOIN native n
CROSS JOIN LATERAL (SELECT CASE WHEN NOT isfinite(f.intercept+f.slope*s.logr+EXP(f.old_t)/2)
 OR ABS(f.intercept+f.slope*s.logr+EXP(f.old_t)/2)>700
 THEN error('normal RE marginal predictor outside numerical bounds') ELSE EXP(f.intercept+f.slope*s.logr+EXP(f.old_t)/2) END AS marginal_mu) projection
ORDER BY s.id,s.year
""" into normal_re_poisson_predictions
sql """WITH RECURSIVE
z_values AS (
 SELECT 'intercept' AS name,ABS(estimate_intercept)/SQRT(covariance_aa) AS z FROM (SELECT * FROM active LIMIT 1)
 UNION ALL SELECT 'slope',ABS(estimate_slope)/SQRT(covariance_bb) FROM (SELECT * FROM active LIMIT 1)
 UNION ALL SELECT 'lnvariance',ABS(estimate_lnvariance)/SQRT(covariance_tt) FROM (SELECT * FROM active LIMIT 1)
 UNION ALL SELECT 'variance_boundary',SQRT(lr_variance_zero) FROM (SELECT * FROM active LIMIT 1)
), small_tail(name,z,iter,term,partial) AS (
 SELECT name,z,0,z/SQRT(2),z/SQRT(2) FROM z_values WHERE z<=3
 UNION ALL SELECT name,z,iter+1,term*(-(z*z/2))*(2*iter+1)/((iter+1)*(2*iter+3)),
   partial+term*(-(z*z/2))*(2*iter+1)/((iter+1)*(2*iter+3)) FROM small_tail WHERE iter<80
), large_tail(name,z,iter,denominator) AS (
 SELECT name,z,100,z FROM z_values WHERE z>3
 UNION ALL SELECT name,z,iter-1,z+iter/denominator FROM large_tail WHERE iter>0
), pvalues AS (
 SELECT name,1-2/SQRT(PI())*partial AS p FROM small_tail WHERE iter=80
 UNION ALL SELECT name,2*EXP(-z*z/2)/SQRT(2*PI())/denominator AS p FROM large_tail WHERE iter=0
)
SELECT MAX(candidate_rows) AS candidate_rows,MAX(firms) AS firms,MAX(zero_total_firms) AS zero_total_firms,
 MAX(estimate_intercept) AS estimate_intercept,MAX(estimate_slope) AS estimate_slope,MAX(estimate_lnvariance) AS estimate_lnvariance,MAX(sigma) AS sigma,
 SQRT(MAX(covariance_aa)) AS se_intercept,SQRT(MAX(covariance_bb)) AS se_slope,SQRT(MAX(covariance_tt)) AS se_lnvariance,
 MAX(sigma)*SQRT(MAX(covariance_tt))/2 AS se_sigma,
 EXP((MAX(estimate_lnvariance)-1.959963984540054*SQRT(MAX(covariance_tt)))/2) AS lower_sigma,
 EXP((MAX(estimate_lnvariance)+1.959963984540054*SQRT(MAX(covariance_tt)))/2) AS upper_sigma,
 MAX(estimate_intercept)/SQRT(MAX(covariance_aa)) AS z_intercept,MAX(estimate_slope)/SQRT(MAX(covariance_bb)) AS z_slope,MAX(estimate_lnvariance)/SQRT(MAX(covariance_tt)) AS z_lnvariance,
 (SELECT p FROM pvalues WHERE name='intercept') AS p_intercept,(SELECT p FROM pvalues WHERE name='slope') AS p_slope,(SELECT p FROM pvalues WHERE name='lnvariance') AS p_lnvariance,
 MAX(estimate_intercept)-1.959963984540054*SQRT(MAX(covariance_aa)) AS lower_intercept,MAX(estimate_intercept)+1.959963984540054*SQRT(MAX(covariance_aa)) AS upper_intercept,
 MAX(estimate_slope)-1.959963984540054*SQRT(MAX(covariance_bb)) AS lower_slope,MAX(estimate_slope)+1.959963984540054*SQRT(MAX(covariance_bb)) AS upper_slope,
 MAX(estimate_lnvariance)-1.959963984540054*SQRT(MAX(covariance_tt)) AS lower_lnvariance,MAX(estimate_lnvariance)+1.959963984540054*SQRT(MAX(covariance_tt)) AS upper_lnvariance,
 MAX(covariance_aa) AS covariance_aa,MAX(covariance_ab) AS covariance_ab,MAX(covariance_at) AS covariance_at,MAX(covariance_bb) AS covariance_bb,MAX(covariance_bt) AS covariance_bt,MAX(covariance_tt) AS covariance_tt,
 MAX(score_intercept) AS score_intercept,MAX(score_slope) AS score_slope,MAX(score_lnvariance) AS score_lnvariance,
 MAX(log_likelihood) AS log_likelihood,MAX(poisson_log_likelihood) AS poisson_log_likelihood,MAX(lr_variance_zero) AS lr_variance_zero,
 (SELECT p/2 FROM pvalues WHERE name='variance_boundary') AS p_lr_variance_zero,
 AVG(marginal_mu ORDER BY id,year) AS mean_mu,MIN(marginal_mu) AS min_mu,MAX(marginal_mu) AS max_mu,AVG(marginal_residual ORDER BY id,year) AS mean_residual,
 MAX(iterations) AS iterations,MAX(last_step) AS last_step,MAX(quad_diff) AS quad_diff,
 'normal_asymptotic' AS inference,'chi_bar_square_0_1_equal_mixture' AS lr_reference FROM active"""
export normal_re_poisson_predictions.csv
