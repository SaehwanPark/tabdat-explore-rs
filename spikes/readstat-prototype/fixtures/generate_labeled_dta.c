/* Generates fixtures/labeled.dta with the ReadStat v1.0.0 writer API.
 *
 * The fixture exercises the ingestion surface the spike must prove:
 * variable labels, numeric and string value-label sets, system missing,
 * tagged missing (.a / .b), and mixed numeric/string storage.
 *
 * Regenerate with (from spikes/readstat-prototype, after `cargo build` has
 * produced the static library under the target directory):
 *
 *   cc -o /tmp/gen_labeled fixtures/generate_labeled_dta.c \
 *     -I<readstat-source>/src \
 *     <target>/readstat-spike/debug/build/<hash>/out/readstat/src/.libs/libreadstat.a \
 *     -lz -liconv
 *   /tmp/gen_labeled fixtures/labeled.dta
 *
 * The committed fixture is the source of truth for the tests; regenerate only
 * to extend the fixture, and update the tests deliberately.
 */
#include <stdio.h>
#include <stdlib.h>
#include "readstat.h"

static ssize_t write_bytes(const void *data, size_t len, void *ctx) {
  FILE *file = (FILE *)ctx;
  return fwrite(data, 1, len, file);
}

static void on_error(const char *message, void *ctx) {
  (void)ctx;
  fprintf(stderr, "readstat writer error: %s\n", message);
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "usage: %s <output.dta>\n", argv[0]);
    return 1;
  }
  FILE *file = fopen(argv[1], "wb");
  if (file == NULL) {
    fprintf(stderr, "cannot open %s\n", argv[1]);
    return 1;
  }

  readstat_writer_t *writer = readstat_writer_init();
  readstat_set_data_writer(writer, write_bytes);
  readstat_writer_set_error_handler(writer, on_error);
  readstat_writer_set_file_label(writer, "TabDat ReadStat spike fixture");

  readstat_label_set_t *band = readstat_add_label_set(writer, READSTAT_TYPE_INT32, "income_band");
  readstat_label_int32_value(band, 1, "low");
  readstat_label_int32_value(band, 2, "mid");
  readstat_label_int32_value(band, 3, "high");

  readstat_label_set_t *sex = readstat_add_label_set(writer, READSTAT_TYPE_STRING, "sex_label");
  readstat_label_string_value(sex, "M", "Male");
  readstat_label_string_value(sex, "F", "Female");

  readstat_variable_t *id = readstat_add_variable(writer, "id", READSTAT_TYPE_INT32, 0);
  readstat_variable_set_label(id, "Subject ID");

  readstat_variable_t *age = readstat_add_variable(writer, "age", READSTAT_TYPE_FLOAT, 0);
  readstat_variable_set_label(age, "Age in years");

  readstat_variable_t *band_var = readstat_add_variable(writer, "band", READSTAT_TYPE_INT32, 0);
  readstat_variable_set_label(band_var, "Income band");
  readstat_variable_set_label_set(band_var, band);

  readstat_variable_t *sex_var = readstat_add_variable(writer, "sex", READSTAT_TYPE_STRING, 1);
  readstat_variable_set_label(sex_var, "Sex");
  readstat_variable_set_label_set(sex_var, sex);

  readstat_variable_t *score = readstat_add_variable(writer, "score", READSTAT_TYPE_DOUBLE, 0);
  readstat_variable_set_label(score, "Test score");

  if (readstat_begin_writing_dta(writer, file, 4) != READSTAT_OK) {
    fprintf(stderr, "begin_writing_dta failed\n");
    return 1;
  }

  /* Row 1: fully populated. */
  readstat_begin_row(writer);
  readstat_insert_int32_value(writer, id, 1);
  readstat_insert_float_value(writer, age, 30.5f);
  readstat_insert_int32_value(writer, band_var, 1);
  readstat_insert_string_value(writer, sex_var, "M");
  readstat_insert_double_value(writer, score, 88.25);
  readstat_end_row(writer);

  /* Row 2: tagged missing .a on score. */
  readstat_begin_row(writer);
  readstat_insert_int32_value(writer, id, 2);
  readstat_insert_float_value(writer, age, 41.0f);
  readstat_insert_int32_value(writer, band_var, 2);
  readstat_insert_string_value(writer, sex_var, "F");
  readstat_insert_tagged_missing_value(writer, score, 'a');
  readstat_end_row(writer);

  /* Row 3: system missing on score, missing string on sex. */
  readstat_begin_row(writer);
  readstat_insert_int32_value(writer, id, 3);
  readstat_insert_float_value(writer, age, 52.5f);
  readstat_insert_int32_value(writer, band_var, 3);
  readstat_insert_missing_value(writer, sex_var);
  readstat_insert_missing_value(writer, score);
  readstat_end_row(writer);

  /* Row 4: tagged missing .b on score. */
  readstat_begin_row(writer);
  readstat_insert_int32_value(writer, id, 4);
  readstat_insert_float_value(writer, age, 63.0f);
  readstat_insert_int32_value(writer, band_var, 1);
  readstat_insert_string_value(writer, sex_var, "M");
  readstat_insert_tagged_missing_value(writer, score, 'b');
  readstat_end_row(writer);

  if (readstat_end_writing(writer) != READSTAT_OK) {
    fprintf(stderr, "end_writing failed\n");
    return 1;
  }
  readstat_writer_free(writer);
  fclose(file);
  return 0;
}
