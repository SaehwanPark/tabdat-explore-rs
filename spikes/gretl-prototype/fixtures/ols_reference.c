#include <stdio.h>
#include <string.h>
#include "libgretl.h"
static const double Y[16] = {60323,61122,60171,61187,63221,63639,64989,63761,66019,67857,68169,66513,68655,69564,69331,70551};
static const double X1[16] = {83.0,88.5,88.2,89.5,96.2,98.1,99.0,100.0,101.2,104.6,108.4,110.8,112.6,114.2,115.7,116.9};
static const double X2[16] = {234289,259426,258054,284599,328975,346999,365385,363112,397469,419180,442769,444546,482704,502601,518173,554894};
static const double X3[16] = {2356,2325,3682,3351,2099,1932,1870,3578,2904,2822,2936,4681,3813,3931,4806,4007};
static const double X4[16] = {1590,1456,1616,1650,3099,3594,3547,3350,3048,2857,2798,2637,2552,2514,2572,2827};
static const double X5[16] = {107608,108632,109773,110929,112075,113270,115094,116219,117388,118734,120445,121950,123366,125368,127852,130081};
static const double X6[16] = {1947,1948,1949,1950,1951,1952,1953,1954,1955,1956,1957,1958,1959,1960,1961,1962};
int main(void) {
    libgretl_init();
    int nvar = 8; /* const(0) + y(1) + x1..x6 (2..7) */
    int nobs = 16;
    DATASET *dset = create_new_dataset(nvar, nobs, 0);
    if (!dset) { fprintf(stderr, "dataset_new failed\n"); return 1; }
    const char *names[8] = {"const","y","x1","x2","x3","x4","x5","x6"};
    for (int i=0;i<8;i++) strcpy(dset->varname[i], names[i]);
    int t;
    for (t=0;t<nobs;t++){ dset->Z[1][t]=Y[t]; dset->Z[2][t]=X1[t]; dset->Z[3][t]=X2[t]; dset->Z[4][t]=X3[t]; dset->Z[5][t]=X4[t]; dset->Z[6][t]=X5[t]; dset->Z[7][t]=X6[t]; }
    int list[9] = {8, 1, 0, 2, 3, 4, 5, 6, 7};
    MODEL *model = gretl_model_new();
    *model = lsq(list, dset, OLS, OPT_Z);
    if (model->errcode) { fprintf(stderr, "estimation error code %d\n", model->errcode); return 1; }
    printf("ncoeff=%d nobs=%d dfn=%d dfd=%d\n", model->ncoeff, model->nobs, model->dfn, model->dfd);
    for (int i=0;i<model->ncoeff;i++) printf("B%d = %.15g  se=%.15g\n", i, model->coeff[i], model->sderr[i]);
    printf("rsq=%.15g adjrsq=%.15g sigma=%.15g ess=%.15g tss=%.15g fstt=%.15g lnL=%.15g\n", model->rsq, model->adjrsq, model->sigma, model->ess, model->tss, model->fstt, model->lnL);
    gretl_model_free(model);
    destroy_dataset(dset);
    libgretl_cleanup();
    return 0;
}
