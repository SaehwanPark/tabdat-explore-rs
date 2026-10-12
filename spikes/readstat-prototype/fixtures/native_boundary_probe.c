/* Native-only ASan/UBSan probe. This does NOT instrument Rust or certify all
 * formats: it exercises the actual pinned DTA parser and pointer bridge. */
#include "readstat.h"
#include "readstat_io_unistd.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int tabdat_readstat_set_value_handlers(readstat_parser_t *);
int tabdat_readstat_value_type(const readstat_value_t *);
const char *tabdat_readstat_string_value(const readstat_value_t *);

typedef struct { int cells, labels, abort_values; } probe_t;

/* Test-only counters: this standalone probe is serial, never the adapter. */
static int read_calls, fail_at;
static int allocations, fail_allocation, frees;

/* Only parser.c/io_unistd.c use these via test compile-time macros. */
void *tabdat_fault_calloc(size_t count, size_t size) {
  if (++allocations == fail_allocation) return NULL;
  return calloc(count, size);
}
void tabdat_fault_free(void *pointer) {
  if (pointer != NULL) frees++;
  free(pointer);
}
static ssize_t injected_read(void *buffer, size_t length, void *ctx) {
  if (++read_calls == fail_at) return -1;
  return unistd_read_handler(buffer, length, ctx);
}

int tabdat_readstat_value_callback(int obs, readstat_variable_t *variable,
    const readstat_value_t *value, void *ctx) {
  probe_t *probe = ctx;
  assert(obs >= 0 && readstat_variable_get_index(variable) >= 0);
  if (tabdat_readstat_value_type(value) == READSTAT_TYPE_STRING) {
    const char *text = tabdat_readstat_string_value(value);
    assert(text != NULL);
    (void)strlen(text);
  }
  probe->cells++;
  return probe->abort_values ? READSTAT_HANDLER_ABORT : READSTAT_HANDLER_OK;
}

int tabdat_readstat_label_callback(const char *set, const readstat_value_t *value,
    const char *label, void *ctx) {
  assert(set != NULL && label != NULL);
  (void)tabdat_readstat_value_type(value);
  ((probe_t *)ctx)->labels++;
  return READSTAT_HANDLER_OK;
}

static int variable_callback(int index, readstat_variable_t *variable,
    const char *labels, void *ctx) {
  (void)labels; (void)ctx;
  assert(index >= 0 && variable != NULL);
  return READSTAT_HANDLER_OK;
}

static void parse(const char *path, int abort_values, int expected_error,
    int expected_cells, int expected_labels) {
  probe_t probe = { .abort_values = abort_values };
  readstat_parser_t *parser = readstat_parser_init();
  assert(parser != NULL);
  assert(tabdat_readstat_set_value_handlers(parser) == READSTAT_OK);
  assert(readstat_set_variable_handler(parser, variable_callback) == READSTAT_OK);
  int code = readstat_parse_dta(parser, path, &probe);
  readstat_parser_free(parser);
  if (expected_error) assert(code != READSTAT_OK);
  else assert(code == READSTAT_OK);
  if (abort_values) assert(code == READSTAT_ERROR_USER_ABORT);
  assert(probe.cells == expected_cells && probe.labels == expected_labels);
}

int main(int argc, char **argv) {
  if (argc != 4) return 2;
  for (int i = 1; i <= 3; i++) {
    allocations = 0; frees = 0; fail_allocation = i;
    assert(readstat_parser_init() == NULL);
    assert(allocations == i && frees == i - 1);
  }
  fail_allocation = -1;
  for (int i = 0; i < 50; i++) {
    parse(argv[1], 0, 0, 20, 5);
    parse(argv[1], 1, 1, 1, 0);
    parse(argv[2], 0, 0, 12, 0);
    parse(argv[3], 0, 1, 0, 0);
  }
  probe_t probe = {0};
  readstat_parser_t *parser = readstat_parser_init();
  assert(parser != NULL);
  assert(tabdat_readstat_set_value_handlers(parser) == READSTAT_OK);
  assert(readstat_set_variable_handler(parser, variable_callback) == READSTAT_OK);
  assert(readstat_set_read_handler(parser, injected_read) == READSTAT_OK);
  fail_at = -1; read_calls = 0;
  assert(readstat_parse_dta(parser, argv[1], &probe) == READSTAT_OK);
  int total = read_calls;
  for (int i = 1; i <= total; i++) {
    read_calls = 0; fail_at = i; probe = (probe_t){0};
    int code = readstat_parse_dta(parser, argv[1], &probe);
    if (code != READSTAT_ERROR_READ) fprintf(stderr, "read fault %d returned %d, expected READ\n", i, code);
    assert(code == READSTAT_ERROR_READ);
  }
  fail_at = -1; read_calls = 0; probe = (probe_t){0};
  assert(readstat_parse_dta(parser, argv[1], &probe) == READSTAT_OK);
  readstat_parser_free(parser);
  printf("3 constructor allocation faults, 200 parses, %d read faults/recovery passed\n", total);
  return 0;
}
