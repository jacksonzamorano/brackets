use std::{collections::HashMap, str::Chars};


macro_rules! skip_whitespace {
    ($c: expr) => {
        if $c == ' ' || $c == '\n' || $c == '\t' {
            continue
        }
    };
}
macro_rules! is_quote {
    ($cv: expr, $it: expr) => {
        $cv.chars().last().unwrap_or('_') != '\\' && $it == '"'
    };
}

struct JsonDecoder;
impl JsonDecoder {
    fn derive_key(enumerator: &mut Chars) -> String {
        let mut current_key = String::new();
        'key: while let Some(key_content) = enumerator.next() {
            if key_content != '"' {
                current_key.push(key_content)
            } else {
                // Skip the colon (and spaces)
                for t in enumerator.by_ref() {
                    if t == ':' { break }
                }
                break 'key;
            }
        }
        current_key
    }

    fn derive_value<T: Iterator<Item = char>>(enumerator: &mut T) -> String {
        let mut current_value = String::new();
        let mut value_start = ' ';
        while value_start == ' ' || value_start == ',' {
            value_start = enumerator.next().unwrap();
        }
        let current_type = JsonType::type_for_delimiter(value_start);
        let mut delimeter_count = 1;
        while let Some(n) = enumerator.next() {
            if delimeter_count == 1 {
                skip_whitespace!(n);
            }
            if current_type.should_increment(current_value.chars().last().unwrap_or('_'), n, delimeter_count) {
                delimeter_count += 1;
            } else if current_type.should_decrement(current_value.chars().last().unwrap_or('_'), n, delimeter_count) {
                delimeter_count -= 1;
            }
            if delimeter_count == 0 && current_type.character_ends_type(n) { break; }
            current_value.push(n);
        }
        current_value
    }
}

/// A JSON structure that is formatted
/// like the following:
///
/// {
///     "key": "value"
/// }
#[derive(Debug)]
pub struct JsonObject {
    keys: HashMap<String, String>,
}

impl JsonObject {

    /// Creates an empty JSON object.
    /// This is useful for building a JSON
    /// object from scratch.
    pub fn empty() -> JsonObject {
        JsonObject {
            keys: HashMap::new()
        }
    }

    /// Builds a JSONObject from a string
    /// containing keys and values.
    ///
    /// # Arguments
    ///
    /// * `json` — An owned string containing the JSON.
    pub fn from_string(json: &str) -> JsonObject {
        let mut keys: HashMap<String, String> = HashMap::new();
        let mut enumerator = json.chars();
        while let Some(c) = enumerator.next() {
            if c == '"' {
                keys.insert(JsonDecoder::derive_key(&mut enumerator), JsonDecoder::derive_value(&mut enumerator));
            }
        }
        JsonObject { keys }
    }

    /// Return a key of the JSON object as a type which
    /// implements JsonRetrieve.
    ///
    /// # Arguments
    ///
    /// * `key` — The key to retrieve from.
    pub fn get<T: JsonRetrieve>(&self, key: &str) -> Result<T, JsonParseError> {
        T::parse(key.to_string(), self.keys.get(key))
    }

    /// Return a key of the JSON object as a type which
    /// implements JsonRetrieve.
    ///
    /// # Arguments
    ///
    /// * `key` — The key to retrieve from.
    pub fn set<T: ToJson>(&mut self, key: &str, data: T) {
        self.keys.insert(key.to_string(), data.to_json());
    }
}
impl Default for JsonObject {
    fn default() -> Self {
        JsonObject::empty()
    }
}

#[derive(Debug)]
pub struct JsonArray {
    values: Vec<String>,
}
impl JsonArray {
    /// Creates an empty JSON array.
    /// This is useful for building a JSON
    /// array from scratch.
    pub fn empty() -> JsonArray {
        JsonArray { values: Vec::new() }
    }

    /// Builds a JSONArray from a string
    /// containing children that implement
    /// `JsonRetreive`
    ///
    /// # Arguments
    ///
    /// * `json` — An owned string containing the JSON.
    pub fn from_string(json: &str) -> JsonArray {
        let mut values: Vec<String> = Vec::new();
        let mut enumerator = json.chars().peekable();

        while enumerator.peek().is_some() {
            values.push(JsonDecoder::derive_value(&mut enumerator));
        }

        JsonArray { values }
    }

    /// Gets the object at the index as a type
    /// that implements JsonRetrieve.
    ///
    /// # Arguments
    ///
    /// * `index` — The index to retrieve from.
    pub fn get<T: JsonRetrieve>(&self, index: usize) -> Result<T, JsonParseError> {
        T::parse(index.to_string(), self.values.get(index))
    }

    /// Converts all elements of this JSONArray
    /// to a type that implements JsonRetrieve.
    /// Progagates errors if any child keys are invalid.
    pub fn map<T: JsonRetrieve>(&self) -> Result<Vec<T>, JsonParseError> {
        if self.values.is_empty() {
            return Ok(Vec::new());
        }
        let mut build = Vec::new();
        for i in 0..self.values.len() {
            let value = &self.values[i];
            build.push(T::parse(i.to_string(), Some(value))?);
        }
        Ok(build)
    }

    /// Converts all elements of this JSONArray
    /// to a type that implements JsonRetrieve.
    /// Silently drops any invalid children.
    pub fn map_drop<T: JsonRetrieve>(&self) -> Vec<T> {
        if self.values.is_empty() {
            return Vec::new();
        }
        let mut build = Vec::new();
        for i in 0..self.values.len() {
            let value = &self.values[i];
            if let Ok(val) = T::parse(i.to_string(), Some(value)) {
                build.push(val);
            }
        }
        build
    }
}
impl Default for JsonArray {
    fn default() -> Self {
        JsonArray::empty()
    }
}

#[derive(Debug, PartialEq)]
enum JsonType {
    Primitive,
    String,
    Object,
    Array,
}

impl JsonType {
    pub fn type_for_delimiter(dlm: char) -> JsonType {
        if dlm == '[' {
            JsonType::Array
        } else if dlm == '{' {
            JsonType::Object
        } else if dlm == '"' {
            JsonType::String
        } else {
            JsonType::Primitive
        }
    }

    fn character_ends_type(&self, c: char) -> bool {
        match self {
            JsonType::Primitive => c == ',' || c == '}' || c == ']',
            JsonType::String => c == ',' || c == '"',
            JsonType::Object => c == ',' || c == '}',
            JsonType::Array => c == ',' || c == ']',
        }
    }

    fn should_increment(&self, prev: char, c: char, count: i32) -> bool {
        match self {
            JsonType::Primitive => false,
            JsonType::String => prev != '\\' && c == '"' && count % 2 == 0,
            JsonType::Object => c == '{',
            JsonType::Array => c == '[',
        }
    }

    fn should_decrement(&self, prev: char, c: char, count: i32) -> bool {
        match self {
            JsonType::Primitive => false,
            JsonType::String => prev != '\\' && c == '"' && count % 2 == 1,
            JsonType::Object => c == '}',
            JsonType::Array => c == ']',
        }
    }
}

#[derive(Debug)]
pub enum JsonParseError {
    NotFound(String),
    InvalidType(String, &'static str),
}

/// ToJson is a trait that allows any conforming
/// structs to convert to a JSON format.
///
/// A default implemenation is most easily
/// obtained by deriving this trait.
pub trait ToJson {
    /// ToJson creates a JSON string from
    /// anything which implements it
    fn to_json(&self) -> String;
}

/// FromJs is a trait that allows any conforming
/// structs to be converted from a JSON format.
///
/// A default implemenation is most easily
/// obtained by deriving this trait.
pub trait FromJson {
    fn from_json(json: &JsonObject) -> Result<Self, JsonParseError>
    where
        Self: Sized;
}

impl ToJson for String {
    fn to_json(&self) -> String {
        let mut o = String::new();
        o += "\"";
        o += &self.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t");
        o += "\"";
        o
    }
}
impl ToJson for str {
    fn to_json(&self) -> String {
        let mut o = String::new();
        o += "\"";
        o += &self.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t");
        o += "\"";
        o
    }
}
impl ToJson for i32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}
impl ToJson for i64 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}
impl ToJson for u32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}
impl ToJson for u64 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}
impl ToJson for f32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}
impl ToJson for f64 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}
impl ToJson for bool {
    fn to_json(&self) -> String {
        if *self {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }
}
impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> String {
        let mut output = String::new();
        output += "[";
        for i in self.iter() {
            output += &i.to_json();
            output += ",";
        }
        if !self.is_empty() {
            output.pop();
        }
        output += "]";
        output
    }
}
impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> String {
        match self {
            Some(x) => x.to_json(),
            None => "null".to_string(),
        }
    }
}
impl<K: ToJson, V: ToJson> ToJson for HashMap<K, V> {
    fn to_json(&self) -> String {
        let mut output = String::new();
        output += "{";
        for (k, v) in self {
            output += "\"";
            output += &k.to_json();
            output += "\":";
            output += &v.to_json();
            output += ",";
        }
        output.pop();
        output += "}";
        output
    }
}
impl ToJson for JsonObject {
    fn to_json(&self) -> String {
        let mut output = "{".to_string();
        for (k, v) in &self.keys {
            output += "\"";
            output += k;
            output += "\":";
            output += v;
            output += ",";
        }
        output.pop();
        output += "}";
        output
    }
}
impl ToJson for JsonArray {
    fn to_json(&self) -> String {
        let mut output = "[".to_string();
        for v in &self.values {
            output += v;
            output += ",";
        }
        output.pop();
        output += "]";
        output
    }
}

pub trait JsonRetrieve {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError>
    where
        Self: Sized;
}

impl JsonRetrieve for String {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        Ok(value.ok_or(JsonParseError::NotFound(key))?.to_string())
    }
}
impl JsonRetrieve for i32 {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        if let Some(v) = value {
            Ok(v.parse().map_err(|_| JsonParseError::InvalidType(key, "i32"))?)
        } else {
            Err(JsonParseError::NotFound(key))
        }
    }
}
impl JsonRetrieve for i64 {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        if let Some(v) = value {
            Ok(v.parse().map_err(|_| JsonParseError::InvalidType(key, "i64"))?)
        } else {
            Err(JsonParseError::NotFound(key))
        }
    }
}
impl JsonRetrieve for f32 {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        if let Some(v) = value {
            Ok(v.parse().map_err(|_| JsonParseError::InvalidType(key, "f32"))?)
        } else {
            Err(JsonParseError::NotFound(key))
        }
    }
}
impl JsonRetrieve for f64 {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        if let Some(v) = value {
            Ok(v.parse().map_err(|_| JsonParseError::InvalidType(key, "f64"))?)
        } else {
            Err(JsonParseError::NotFound(key))
        }
    }
}
impl JsonRetrieve for bool {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError>  {
        if let Some(v) = value {
            match v.as_ref() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(JsonParseError::InvalidType(key, "bool")),
            }
        } else {
            Err(JsonParseError::NotFound(key))
        }
    }
}
impl<T: JsonRetrieve> JsonRetrieve for Vec<T> {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        JsonArray::from_string(value.ok_or(JsonParseError::NotFound(key))?).map()
    }
}
impl<T: JsonRetrieve> JsonRetrieve for Option<T> {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        if let Some(v) = value {
            if v != "null" {
                return Ok(Some(T::parse(key, value)?));
            }
        }
        Ok(None)
    }
}
impl JsonRetrieve for JsonObject {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        Ok(JsonObject::from_string(value.ok_or(JsonParseError::NotFound(key))?))
    }
}
impl JsonRetrieve for JsonArray {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        Ok(JsonArray::from_string(value.ok_or(JsonParseError::NotFound(key))?))
    }
}
impl<T: FromJson> JsonRetrieve for T {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        Self::from_json(&JsonObject::from_string(value.ok_or(JsonParseError::NotFound(key))?))
    }
}

#[cfg(chrono)]
use chrono::{DateTime, Utc};

#[cfg(chrono)]
impl JsonRetrieve for DateTime<Utc> {
    fn parse(key: String, value: Option<&String>) -> Result<Self, JsonParseError> {
        if let Some(v) = value {
            Ok(DateTime::parse_from_rfc3339(&v.replace('\"', ""))
                .map_err(|_| JsonParseError::InvalidType(key, "RFC3339 Date"))?
                .with_timezone(&Utc))
        } else {
            Err(JsonParseError::NotFound(key))
        }
    }
}

#[cfg(chrono)]
impl ToJson for DateTime<Utc> {
    fn to_json(&self) -> String {
        format!("\"{}\"", self.to_rfc3339())
    }
}
