use crate::*;
use std::fmt::Write as _;

pub(super) fn duplicate_count_alias(variables: &[String]) -> String {
  let base = "__tabdat_duplicate_count";
  let mut suffix = 0_u64;
  loop {
    let candidate = if suffix == 0 {
      base.to_owned()
    } else {
      format!("{base}_{suffix}")
    };
    if variables
      .iter()
      .all(|variable| !variable.eq_ignore_ascii_case(&candidate))
    {
      return quote_identifier(&candidate);
    }
    suffix = suffix
      .checked_add(1)
      .expect("duplicate alias suffix overflow");
  }
}

pub(super) fn isid_count_alias(variables: &[String]) -> String {
  let base = "__tabdat_isid_count";
  let mut suffix = 0_u64;
  loop {
    let candidate = if suffix == 0 {
      base.to_owned()
    } else {
      format!("{base}_{suffix}")
    };
    if variables
      .iter()
      .all(|variable| !variable.eq_ignore_ascii_case(&candidate))
    {
      return quote_identifier(&candidate);
    }
    suffix = suffix.checked_add(1).expect("isid alias suffix overflow");
  }
}

pub(super) fn update_signature_token(digest: &mut Sha256, token: &[u8]) {
  let length = (token.len() as u64).to_be_bytes();
  digest.update(length);
  digest.update(token);
}

pub(super) fn signature_text(value: &str) -> Vec<u8> {
  let mut encoded = Vec::with_capacity(value.len() + 1);
  encoded.push(b'T');
  encoded.extend_from_slice(value.as_bytes());
  encoded
}

pub(super) fn signature_value(
  value: &Value,
  type_hint: Option<&str>,
) -> Result<Vec<u8>, DataError> {
  match value {
    Value::Null => Ok(b"N".to_vec()),
    Value::Boolean(value) => Ok(if *value { b"B1" } else { b"B0" }.to_vec()),
    Value::TinyInt(value) => Ok(integer_value(*value)),
    Value::SmallInt(value) => Ok(integer_value(*value)),
    Value::Int(value) => Ok(integer_value(*value)),
    Value::BigInt(value) => Ok(integer_value(*value)),
    Value::HugeInt(value) => Ok(integer_value(*value)),
    Value::UHugeInt(value) => Ok(integer_value(*value)),
    Value::UTinyInt(value) => Ok(integer_value(*value)),
    Value::USmallInt(value) => Ok(integer_value(*value)),
    Value::UInt(value) => Ok(integer_value(*value)),
    Value::UBigInt(value) => Ok(integer_value(*value)),
    Value::Float(value) => Ok(float_value(f64::from(*value))),
    Value::Double(value) => Ok(float_value(*value)),
    Value::Decimal(value) => Ok(text_value(b'D', &value.to_string())),
    Value::Timestamp(unit, value) => Ok(text_value(
      b'Z',
      &format_timestamp(*unit, *value, is_timezone_type(type_hint)),
    )),
    Value::Text(value) => Ok(text_value(b'S', value)),
    Value::Blob(value) | Value::Geometry(value) => {
      let mut encoded = Vec::with_capacity(value.len() + 1);
      encoded.push(b'Y');
      encoded.extend_from_slice(value);
      Ok(encoded)
    }
    Value::Date32(value) => Ok(text_value(b'A', &format_date(i64::from(*value)))),
    Value::Time64(unit, value) => Ok(text_value(b'H', &format_time(*unit, *value))),
    Value::List(values) | Value::Array(values) => sequence_value(values, type_hint),
    Value::Enum(value) => Ok(text_value(b'S', value)),
    Value::Struct(values) => struct_value(values.iter(), type_hint),
    Value::Map(values) => map_sequence_value(values, type_hint),
    Value::Interval {
      months,
      days,
      nanos,
    } => sequence_value(
      &[
        Value::HugeInt(i128::from(*months)),
        Value::HugeInt(i128::from(*days)),
        Value::HugeInt(i128::from(*nanos)),
      ],
      None,
    ),
    Value::Union(_) => Err(DataError),
    _ => Err(DataError),
  }
}

pub(super) fn integer_value<T: fmt::Display>(value: T) -> Vec<u8> {
  let mut encoded = String::from("I");
  write!(&mut encoded, "{value}").expect("writing to a String cannot fail");
  encoded.into_bytes()
}

pub(super) fn text_value(prefix: u8, value: &str) -> Vec<u8> {
  let mut encoded = Vec::with_capacity(value.len() + 1);
  encoded.push(prefix);
  encoded.extend_from_slice(value.as_bytes());
  encoded
}

pub(super) fn float_value(value: f64) -> Vec<u8> {
  text_value(b'F', &python_float_hex(value))
}

pub fn python_float_hex(value: f64) -> String {
  if value.is_nan() {
    return "nan".to_owned();
  }
  if value == f64::INFINITY {
    return "inf".to_owned();
  }
  if value == f64::NEG_INFINITY {
    return "-inf".to_owned();
  }

  let bits = value.to_bits();
  let sign = if bits >> 63 == 1 { "-" } else { "" };
  let magnitude = bits & 0x7fff_ffff_ffff_ffff;
  if magnitude == 0 {
    return format!("{sign}0x0.0p+0");
  }

  let exponent_bits = (magnitude >> 52) & 0x7ff;
  let fraction = magnitude & ((1_u64 << 52) - 1);
  if exponent_bits == 0 {
    format!("{sign}0x0.{fraction:013x}p-1022")
  } else {
    let exponent = i32::try_from(exponent_bits).expect("11-bit exponent fits i32") - 1023;
    format!("{sign}0x1.{fraction:013x}p{exponent:+}")
  }
}

pub(super) fn sequence_value(
  values: &[Value],
  type_hint: Option<&str>,
) -> Result<Vec<u8>, DataError> {
  let mut encoded = b"L".to_vec();
  append_ascii_length(&mut encoded, values.len())?;
  let child_hint = list_value_type(type_hint);
  for value in values {
    let item = signature_value(value, child_hint)?;
    append_signature_part(&mut encoded, &item)?;
  }
  Ok(encoded)
}

pub(super) fn struct_value<'a, I>(values: I, type_hint: Option<&str>) -> Result<Vec<u8>, DataError>
where
  I: IntoIterator<Item = &'a (String, Value)>,
{
  let mut entries = values
    .into_iter()
    .map(|(key, value)| {
      let key_value = Value::Text(key.clone());
      let value_hint = struct_field_type(type_hint, key);
      Ok::<_, DataError>((
        signature_value(&key_value, None)?,
        signature_value(value, value_hint.as_deref())?,
      ))
    })
    .collect::<Result<Vec<_>, _>>()?;
  mapping_entries(&mut entries)
}

pub(super) fn map_sequence_value(
  values: &duckdb::types::OrderedMap<Value, Value>,
  type_hint: Option<&str>,
) -> Result<Vec<u8>, DataError> {
  let type_hints = map_value_types(type_hint);
  let key_hint = type_hints.as_ref().map(|(key, _)| key.as_str());
  let value_hint = type_hints.as_ref().map(|(_, value)| value.as_str());
  let mut encoded = b"L".to_vec();
  append_ascii_length(&mut encoded, values.iter().count())?;
  for (key, value) in values.iter() {
    let pair = sequence_pair_value(key, value, key_hint, value_hint)?;
    append_signature_part(&mut encoded, &pair)?;
  }
  Ok(encoded)
}

pub(super) fn sequence_pair_value(
  key: &Value,
  value: &Value,
  key_hint: Option<&str>,
  value_hint: Option<&str>,
) -> Result<Vec<u8>, DataError> {
  let mut encoded = b"L".to_vec();
  append_ascii_length(&mut encoded, 2)?;
  let key = signature_value(key, key_hint)?;
  let value = signature_value(value, value_hint)?;
  append_signature_part(&mut encoded, &key)?;
  append_signature_part(&mut encoded, &value)?;
  Ok(encoded)
}

pub(super) fn mapping_entries(entries: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<Vec<u8>, DataError> {
  entries.sort();

  let mut encoded = b"M".to_vec();
  append_ascii_length(&mut encoded, entries.len())?;
  for (key, value) in entries {
    append_signature_part(&mut encoded, key)?;
    append_signature_part(&mut encoded, value)?;
  }
  Ok(encoded)
}

pub(super) fn list_value_type(type_hint: Option<&str>) -> Option<&str> {
  let type_hint = type_hint?.trim();
  if let Some(value) = type_hint
    .strip_prefix("LIST(")
    .and_then(|value| value.strip_suffix(')'))
  {
    return Some(value.trim());
  }
  type_hint.strip_suffix("[]").map(str::trim)
}

pub(super) fn map_value_types(type_hint: Option<&str>) -> Option<(String, String)> {
  let type_hint = type_hint?;
  let body = type_hint
    .strip_prefix("MAP(")
    .and_then(|value| value.strip_suffix(')'))?;
  let (key, value) = split_top_level_once(body)?;
  Some((key.trim().to_owned(), value.trim().to_owned()))
}

pub(super) fn struct_field_type(type_hint: Option<&str>, key: &str) -> Option<String> {
  let type_hint = type_hint?;
  let body = type_hint
    .strip_prefix("STRUCT(")
    .and_then(|value| value.strip_suffix(')'))?;
  for field in split_top_level_parts(body) {
    let field = field.trim();
    if field.is_empty() {
      continue;
    }
    let (field_name, field_type) = if let Some(rest) = field.strip_prefix('"') {
      let (field_name, field_type) = parse_quoted_struct_field(rest)?;
      (field_name, field_type)
    } else {
      let (field_name, field_type) = field.split_once(char::is_whitespace)?;
      (field_name.to_owned(), field_type.trim())
    };
    if field_name.eq_ignore_ascii_case(key) {
      return (!field_type.is_empty()).then(|| field_type.to_owned());
    }
  }
  None
}

pub(super) fn is_timezone_type(type_hint: Option<&str>) -> bool {
  type_hint.is_some_and(|hint| canonical_signature_type(hint).ends_with("_TZ"))
}

pub(super) fn parse_quoted_struct_field(value: &str) -> Option<(String, &str)> {
  let mut name = String::new();
  let mut offset = 0;
  while offset < value.len() {
    let character = value[offset..].chars().next()?;
    let width = character.len_utf8();
    if character == '"' {
      let after_quote = offset + width;
      if value[after_quote..].starts_with('"') {
        name.push('"');
        offset = after_quote + 1;
        continue;
      }
      return Some((name, value[after_quote..].trim()));
    }
    name.push(character);
    offset += width;
  }
  None
}

pub(super) fn split_top_level_once(value: &str) -> Option<(&str, &str)> {
  let mut depth = 0_u32;
  let mut quoted = false;
  let mut characters = value.char_indices().peekable();
  while let Some((index, character)) = characters.next() {
    match character {
      '"' => {
        if quoted && characters.peek().is_some_and(|(_, next)| *next == '"') {
          characters.next();
        } else {
          quoted = !quoted;
        }
      }
      '(' if !quoted => depth = depth.checked_add(1)?,
      ')' if !quoted => depth = depth.checked_sub(1)?,
      ',' if !quoted && depth == 0 => return Some((&value[..index], &value[index + 1..])),
      _ => {}
    }
  }
  None
}

pub(super) fn split_top_level_parts(value: &str) -> Vec<&str> {
  let mut parts = Vec::new();
  let mut start = 0;
  let mut depth = 0_u32;
  let mut quoted = false;
  let mut characters = value.char_indices().peekable();
  while let Some((index, character)) = characters.next() {
    match character {
      '"' => {
        if quoted && characters.peek().is_some_and(|(_, next)| *next == '"') {
          characters.next();
        } else {
          quoted = !quoted;
        }
      }
      '(' if !quoted => depth = depth.saturating_add(1),
      ')' if !quoted => depth = depth.saturating_sub(1),
      ',' if !quoted && depth == 0 => {
        parts.push(&value[start..index]);
        start = index + character.len_utf8();
      }
      _ => {}
    }
  }
  parts.push(&value[start..]);
  parts
}

pub(super) fn append_ascii_length(output: &mut Vec<u8>, length: usize) -> Result<(), DataError> {
  let length = u64::try_from(length).map_err(|_| DataError)?;
  output.extend_from_slice(length.to_string().as_bytes());
  Ok(())
}

pub(super) fn append_signature_part(output: &mut Vec<u8>, value: &[u8]) -> Result<(), DataError> {
  let length = u64::try_from(value.len())
    .map_err(|_| DataError)?
    .to_be_bytes();
  output.extend_from_slice(&length);
  output.extend_from_slice(value);
  Ok(())
}

pub(super) fn format_date(days_since_epoch: i64) -> String {
  let (year, month, day) = civil_from_days(days_since_epoch);
  format!("{year:04}-{month:02}-{day:02}")
}

pub(super) fn format_timestamp(unit: TimeUnit, value: i64, timezone: bool) -> String {
  let (seconds, nanos) = split_time(unit, value);
  let days = seconds.div_euclid(86_400);
  let seconds_of_day = seconds.rem_euclid(86_400);
  let hour = seconds_of_day / 3_600;
  let minute = (seconds_of_day % 3_600) / 60;
  let second = seconds_of_day % 60;
  let (year, month, day) = civil_from_days(days);
  let micros = nanos / 1_000;
  let mut formatted = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
  if nanos != 0 {
    if unit == TimeUnit::Nanosecond && nanos % 1_000 != 0 {
      write!(&mut formatted, ".{nanos:09}").expect("writing to a String cannot fail");
    } else if micros != 0 {
      write!(&mut formatted, ".{micros:06}").expect("writing to a String cannot fail");
    }
  }
  if timezone {
    formatted.push_str("+00:00");
  }
  formatted
}

pub(super) fn format_time(unit: TimeUnit, value: i64) -> String {
  let (seconds, nanos) = split_time(unit, value);
  let total_micros =
    (i128::from(seconds) * 1_000_000 + i128::from(nanos) / 1_000).rem_euclid(86_400_000_000);
  let total_micros = i64::try_from(total_micros).expect("time microseconds fit i64");
  let hour = total_micros / 3_600_000_000;
  let minute = (total_micros % 3_600_000_000) / 60_000_000;
  let second = (total_micros % 60_000_000) / 1_000_000;
  let micros = total_micros % 1_000_000;
  if micros == 0 {
    format!("{hour:02}:{minute:02}:{second:02}")
  } else {
    format!("{hour:02}:{minute:02}:{second:02}.{micros:06}")
  }
}

pub(super) fn split_time(unit: TimeUnit, value: i64) -> (i64, i64) {
  let (divisor, nanos_per_unit) = match unit {
    TimeUnit::Second => (1_i128, 1_000_000_000_i128),
    TimeUnit::Millisecond => (1_000_i128, 1_000_000_i128),
    TimeUnit::Microsecond => (1_000_000_i128, 1_000_i128),
    TimeUnit::Nanosecond => (1_000_000_000_i128, 1_i128),
  };
  let value = i128::from(value);
  let seconds = value.div_euclid(divisor);
  let remainder = value.rem_euclid(divisor);
  (
    i64::try_from(seconds).expect("DuckDB timestamp seconds fit i64"),
    i64::try_from(remainder * nanos_per_unit).expect("timestamp nanoseconds fit i64"),
  )
}

pub(super) fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
  let shifted = days_since_epoch + 719_468;
  let era = if shifted >= 0 {
    shifted / 146_097
  } else {
    (shifted - 146_096) / 146_097
  };
  let day_of_era = shifted - era * 146_097;
  let year_of_era =
    (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
  let year = year_of_era + era * 400;
  let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
  let month_part = (5 * day_of_year + 2) / 153;
  let day = day_of_year - (153 * month_part + 2) / 5 + 1;
  let month = month_part + if month_part < 10 { 3 } else { -9 };
  let year = year + if month <= 2 { 1 } else { 0 };
  (
    year,
    u32::try_from(month).expect("civil month is positive"),
    u32::try_from(day).expect("civil day is positive"),
  )
}

pub(super) fn canonical_append_type(data_type: &str) -> String {
  let normalized = data_type
    .chars()
    .filter(|character| !character.is_whitespace())
    .flat_map(|character| character.to_uppercase())
    .collect::<String>();
  let alias = match normalized.as_str() {
    "INT8" => Some("TINYINT"),
    "INT16" => Some("SMALLINT"),
    "INT32" => Some("INTEGER"),
    "INT64" => Some("BIGINT"),
    "UINT8" => Some("UTINYINT"),
    "UINT16" => Some("USMALLINT"),
    "UINT32" => Some("UINTEGER"),
    "UINT64" => Some("UBIGINT"),
    "UINT128" => Some("UHUGEINT"),
    "FLOAT32" => Some("FLOAT"),
    "FLOAT64" => Some("DOUBLE"),
    "TEXT" | "STRING" => Some("VARCHAR"),
    "BOOL" => Some("BOOLEAN"),
    _ => None,
  };
  if let Some(alias) = alias {
    alias.to_owned()
  } else {
    normalized
  }
}

pub(super) fn canonical_signature_type(data_type: &str) -> String {
  let normalized = data_type
    .chars()
    .filter(|character| !character.is_whitespace())
    .flat_map(|character| character.to_uppercase())
    .collect::<String>();
  let alias = match normalized.as_str() {
    "TINYINT" => Some("INT8"),
    "SMALLINT" => Some("INT16"),
    "INTEGER" => Some("INT32"),
    "BIGINT" => Some("INT64"),
    "HUGEINT" => Some("INT128"),
    "UTINYINT" => Some("UINT8"),
    "USMALLINT" => Some("UINT16"),
    "UINTEGER" => Some("UINT32"),
    "UBIGINT" => Some("UINT64"),
    "UHUGEINT" => Some("UINT128"),
    "FLOAT" | "REAL" => Some("FLOAT32"),
    "DOUBLE" => Some("FLOAT64"),
    "BOOLEAN" | "BOOL" => Some("BOOL"),
    "VARCHAR" | "TEXT" | "CATEGORICAL" | "ENUM" => Some("STRING"),
    "DATETIME" | "TIMESTAMP" => Some("TIMESTAMP_US"),
    "TIMESTAMPTZ" | "TIMESTAMPWITHTIMEZONE" => Some("TIMESTAMP_US_TZ"),
    _ => None,
  };
  if let Some(alias) = alias {
    return alias.to_owned();
  }
  if normalized.starts_with("DECIMAL(PRECISION=")
    && let Some((precision, scale)) = parse_decimal_precision_scale(&normalized)
  {
    return format!("DECIMAL({precision},{scale})");
  }
  if normalized.starts_with("DECIMAL(") && normalized.ends_with(')') {
    return normalized;
  }
  if normalized.starts_with("DATETIME(TIME_UNIT=") {
    let unit_start = "DATETIME(TIME_UNIT=".len();
    if let Some(unit_end) = normalized[unit_start..].find([',', ')']) {
      let unit = &normalized[unit_start..unit_start + unit_end];
      let timezone = if normalized
        .split_once("TIME_ZONE=")
        .is_some_and(|(_, value)| !value.starts_with("NONE"))
      {
        "_TZ"
      } else {
        ""
      };
      return format!("TIMESTAMP_{unit}{timezone}");
    }
  }
  if normalized.starts_with("LIST(") && normalized.ends_with(')') {
    return format!(
      "LIST({})",
      canonical_signature_type(&normalized[5..normalized.len() - 1])
    );
  }
  if normalized.ends_with("[]") {
    return format!(
      "LIST({})",
      canonical_signature_type(&normalized[..normalized.len() - 2])
    );
  }
  normalized
}

pub(super) fn parse_decimal_precision_scale(data_type: &str) -> Option<(&str, &str)> {
  let body = data_type
    .strip_prefix("DECIMAL(PRECISION=")?
    .strip_suffix(')')?;
  let (precision, scale) = body.split_once(",SCALE=")?;
  if precision
    .chars()
    .all(|character| character.is_ascii_digit())
    && scale.chars().all(|character| character.is_ascii_digit())
  {
    Some((precision, scale))
  } else {
    None
  }
}

pub(super) fn lower_hex(bytes: &[u8]) -> String {
  const HEX: &[u8; 16] = b"0123456789abcdef";
  let mut encoded = String::with_capacity(bytes.len() * 2);
  for byte in bytes {
    encoded.push(char::from(HEX[usize::from(byte >> 4)]));
    encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
  }
  encoded
}

pub(super) fn cell_value_from_ref(value: ValueRef<'_>) -> Result<CellValue, DataError> {
  match value {
    ValueRef::Null => Ok(CellValue::Null),
    ValueRef::Boolean(value) => Ok(CellValue::Boolean(value)),
    ValueRef::TinyInt(value) => Ok(CellValue::SignedInteger(i128::from(value))),
    ValueRef::SmallInt(value) => Ok(CellValue::SignedInteger(i128::from(value))),
    ValueRef::Int(value) => Ok(CellValue::SignedInteger(i128::from(value))),
    ValueRef::BigInt(value) => Ok(CellValue::SignedInteger(i128::from(value))),
    ValueRef::HugeInt(value) => Ok(CellValue::SignedInteger(value)),
    ValueRef::UTinyInt(value) => Ok(CellValue::UnsignedInteger(u128::from(value))),
    ValueRef::USmallInt(value) => Ok(CellValue::UnsignedInteger(u128::from(value))),
    ValueRef::UInt(value) => Ok(CellValue::UnsignedInteger(u128::from(value))),
    ValueRef::UBigInt(value) => Ok(CellValue::UnsignedInteger(u128::from(value))),
    ValueRef::UHugeInt(value) => Ok(CellValue::UnsignedInteger(value)),
    ValueRef::Float(value) => Ok(CellValue::Float(f64::from(value))),
    ValueRef::Double(value) => Ok(CellValue::Float(value)),
    ValueRef::Decimal(value) => Ok(CellValue::Decimal {
      width: value.width(),
      scale: value.scale(),
      value: value.value(),
    }),
    ValueRef::Text(value) => String::from_utf8(value.to_vec())
      .map(CellValue::Text)
      .map_err(|_| DataError),
    ValueRef::Blob(value) => Ok(CellValue::Bytes(value.to_vec())),
    _ => Err(DataError),
  }
}

pub(super) fn named_table_identifier(table_name: &str) -> String {
  format!("__tabdat_named_{table_name}")
}

pub(super) fn unique_internal_name(candidate: &str, used_names: &HashSet<String>) -> String {
  let normalized: HashSet<String> = used_names.iter().map(|n| n.to_ascii_lowercase()).collect();
  if !normalized.contains(&candidate.to_ascii_lowercase()) {
    return candidate.to_owned();
  }
  let mut counter = 2;
  loop {
    let name = format!("{candidate}_{counter}");
    if !normalized.contains(&name.to_ascii_lowercase()) {
      return name;
    }
    counter += 1;
  }
}

pub(super) fn unique_output_name(candidate: &str, used_names: &HashSet<String>) -> String {
  if !used_names.contains(candidate) {
    return candidate.to_owned();
  }
  let mut counter = 2;
  loop {
    let name = format!("{candidate}_{counter}");
    if !used_names.contains(&name) {
      return name;
    }
    counter += 1;
  }
}

pub fn validate_sql_query(query: &str) -> Result<(), RuntimeError> {
  let first_word = query
    .split_whitespace()
    .next()
    .unwrap_or("")
    .to_ascii_lowercase();
  if first_word != "select" && first_word != "with" {
    return Err(RuntimeError::SqlNotSelectOrWith);
  }
  Ok(())
}
