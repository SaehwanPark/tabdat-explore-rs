#include "readstat.h"

/* These Rust entry points copy values synchronously and never retain pointers.
 * Only this translation unit handles ReadStat's by-value aggregate ABI. */
extern int tabdat_readstat_value_callback(int, readstat_variable_t *,
    const readstat_value_t *, void *);
extern int tabdat_readstat_label_callback(const char *, const readstat_value_t *,
    const char *, void *);

static int value_callback(int obs, readstat_variable_t *variable,
    readstat_value_t value, void *ctx) {
  return tabdat_readstat_value_callback(obs, variable, &value, ctx);
}

static int label_callback(const char *set, readstat_value_t value,
    const char *label, void *ctx) {
  return tabdat_readstat_label_callback(set, &value, label, ctx);
}

int tabdat_readstat_set_value_handlers(readstat_parser_t *parser) {
  int code = readstat_set_value_handler(parser, value_callback);
  if (code != READSTAT_OK) return code;
  return readstat_set_value_label_handler(parser, label_callback);
}

int tabdat_readstat_value_type(const readstat_value_t *value) {
  return readstat_value_type(*value);
}
int tabdat_readstat_value_is_system_missing(const readstat_value_t *value) {
  return readstat_value_is_system_missing(*value);
}
int tabdat_readstat_value_is_tagged_missing(const readstat_value_t *value) {
  return readstat_value_is_tagged_missing(*value);
}
int tabdat_readstat_value_is_defined_missing(const readstat_value_t *value,
    readstat_variable_t *variable) {
  return readstat_value_is_defined_missing(*value, variable);
}
char tabdat_readstat_value_tag(const readstat_value_t *value) {
  return readstat_value_tag(*value);
}
int8_t tabdat_readstat_int8_value(const readstat_value_t *value) {
  return readstat_int8_value(*value);
}
int16_t tabdat_readstat_int16_value(const readstat_value_t *value) {
  return readstat_int16_value(*value);
}
int32_t tabdat_readstat_int32_value(const readstat_value_t *value) {
  return readstat_int32_value(*value);
}
float tabdat_readstat_float_value(const readstat_value_t *value) {
  return readstat_float_value(*value);
}
double tabdat_readstat_double_value(const readstat_value_t *value) {
  return readstat_double_value(*value);
}
const char *tabdat_readstat_string_value(const readstat_value_t *value) {
  return readstat_string_value(*value);
}

/* Bounded test driver: native typed values traverse the same pointer accessors.
 * Return immediately on abort, as the DTA reader does. No pointers survive. */
int tabdat_readstat_exercise_values(int (*handler)(const readstat_value_t *, void *),
    void *ctx) {
  readstat_value_t values[8] = {0};
  values[0].type = READSTAT_TYPE_INT8; values[0].v.i8_value = -7;
  values[1].type = READSTAT_TYPE_INT16; values[1].v.i16_value = -320;
  values[2].type = READSTAT_TYPE_INT32; values[2].v.i32_value = -70000;
  values[3].type = READSTAT_TYPE_FLOAT; values[3].v.float_value = 1.25f;
  values[4].type = READSTAT_TYPE_DOUBLE; values[4].v.double_value = -2.5;
  char text[] = "owned";
  values[5].type = READSTAT_TYPE_STRING; values[5].v.string_value = text;
  values[6].type = READSTAT_TYPE_DOUBLE; values[6].is_system_missing = 1;
  values[7].type = READSTAT_TYPE_DOUBLE;
  values[7].is_tagged_missing = 1; values[7].tag = 'z';
  for (int i = 0; i < 8; i++) {
    int code = handler(&values[i], ctx);
    if (code != READSTAT_HANDLER_OK) return code;
  }
  text[0] = 'X'; /* Rust's owned copy must remain unchanged. */
  return READSTAT_HANDLER_OK;
}
