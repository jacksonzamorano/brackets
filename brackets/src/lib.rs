mod json;
pub use brackets_macros::{FromJson, ToJson};
pub use json::{FromJson, JsonArray, JsonObject, JsonParseError, ToJson};

#[cfg(test)]
mod test {

	use crate::{self as brackets};
	use crate::{FromJson, JsonObject};

	const TEST_JSON: &'static str = r#"
		{
			"string_value": "Some Text Here",
			"number_value": 1,
			"object_value": {
				"string_value": "Some Text Here 2",
				"number_value": 2
			},
			"array_value": [
				{
					"object_value": {
						"string_value": "Some Text Here 3",
						"number_value": 3
					}
				},
				{
					"object_value": {
						"string_value": "Some Text Here 4",
						"number_value": 4
					}
				}
			],
			"array_strings": [
				"test0", "test1",
				"test2"
			],
			"array_nums": [
				0, 1,
				2, 3,
				4
			],
			"array_bools": [ true, false ],
			"string_with_quotes": "Hello \"World!\""
		}
	"#;

	#[derive(FromJson)]
	struct Test {
	    string_value: String,
	    number_value: i32,
	    object_value: NestedTest,
	    array_value: Vec<NestedTestVecItem>,
	    array_strings: Vec<String>,
	    array_nums: Vec<i32>,
	    array_bools: Vec<bool>,
	    string_with_quotes: String
	}

	#[derive(FromJson)]
	struct NestedTestVecItem {
		object_value: NestedTest,
	}

	#[derive(FromJson)]
	struct NestedTest {
	    string_value: String,
	    number_value: i32,
	}

	#[test]
	fn string_value() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    assert_eq!(test.string_value, "Some Text Here")
	}

	#[test]
	fn number_value() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    assert_eq!(test.number_value, 1)
	}

	#[test]
	fn nested_string_value() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    assert_eq!(test.object_value.string_value, "Some Text Here 2")
	}

	#[test]
	fn nested_number_value() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    assert_eq!(test.object_value.number_value, 2)
	}

	#[test]
	fn array_value() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    let mut i = 3;
	    for t in test.array_value {
	    	assert_eq!(t.object_value.string_value, format!("Some Text Here {}", i));
	    	assert_eq!(t.object_value.number_value, i);
	    	i += 1;
	    }
	}

	#[test]
	fn array_nums() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    let mut i = 0;
	    for t in test.array_nums {
	    	assert_eq!(t, i);
	    	i += 1;
	    }
	}

	#[test]
	fn array_strings() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    let mut i = 0;
	    for t in test.array_strings {
	    	assert_eq!(t, format!("test{}", i));
	    	i += 1;
	    }
	}

	#[test]
	fn array_bools() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    assert_eq!(test.array_bools[0], true);
	    assert_eq!(test.array_bools[1], false);
	}

	#[test]
	fn string_with_quotes() {
	    let test = Test::from_json(&JsonObject::from_string(&TEST_JSON)).unwrap();
	    assert_eq!(test.string_with_quotes, "Hello \"World!\"")
	}
}