/* Source-only replacement for gretl 2026b gretl_utils.c discovery fallback.
 * Same GPL context as original source; build.rs validates pinned source first.
 * Dynamic OpenBLAS/MKL/BLIS detection still runs before this fallback. */
static int detect_blas_via_ldd (void)
{
#ifdef __APPLE__
    /* This source-build route links Accelerate, verified separately. Do not
     * probe a fictitious gretlcli or disable normal BLAS/thread initialization. */
    return BLAS_VECLIB;
#else
    int variant = BLAS_UNKNOWN;
    gchar *sout = NULL, *errout = NULL, *targ = NULL;
    gint status = 0;
    GError *gerr = NULL;
    const char *override = getenv("GRETL_LIB_NAME");
    if (override != NULL && *override != '\0') {
        /* Preserve upstream explicit diagnostic override without mutating env. */
        targ = g_strdup(override);
    } else {
#ifdef __linux__
        /* Resolve in the embedding parent: /proc/self/exe passed literally to
         * ldd would resolve ldd's own executable, not this process. */
        targ = g_file_read_link("/proc/self/exe", &gerr);
#else
        targ = g_strdup(GRETL_PREFIX "/lib/libgretl-1.0.so");
#endif
    }
    if (targ == NULL) {
        if (gerr != NULL) {
            fprintf(stderr, "BLAS executable discovery: %s\n", gerr->message);
            g_error_free(gerr);
        }
        return BLAS_UNKNOWN;
    }
    gchar *argv[] = {"ldd", targ, NULL};
    g_spawn_sync(NULL, argv, NULL, G_SPAWN_SEARCH_PATH, NULL, NULL,
                 &sout, &errout, &status, &gerr);
    if (gerr != NULL) {
        fprintf(stderr, "%s\n", gerr->message);
        g_error_free(gerr);
    } else if (status != 0) {
        fprintf(stderr, "%s exited with status %d\n", argv[0], status);
    } else if (sout != NULL) {
        variant = parse_ldd_output(sout);
    } else {
        fprintf(stderr, "%s: Got no output\n", argv[0]);
    }
    g_free(targ); g_free(sout); g_free(errout);
    return variant;
#endif
}
