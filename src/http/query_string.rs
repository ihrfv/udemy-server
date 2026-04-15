use std::collections::HashMap;

#[derive(Debug)]
pub struct QueryString<'buf> {
    data: HashMap<&'buf str, Value<'buf>>,
}

#[derive(Debug, PartialEq)]
pub enum Value<'buf> {
    Single(&'buf str),
    Multiple(Vec<&'buf str>),
}

impl<'buf> QueryString<'buf> {
    pub fn get(&self, key: &str) -> Option<&Value<'_>> {
        self.data.get(key)
    }
}

impl<'buf> From<&'buf str> for QueryString<'buf> {
    fn from(value: &'buf str) -> Self {
        let mut data = HashMap::new();

        for sub_str in value.split('&').filter(|s| !s.is_empty()) {
            let mut key = sub_str;
            let mut value = "";

            if let Some(i) = sub_str.find('=') {
                key = &sub_str[..i];
                value = &sub_str[i + 1..];
            }

            data.entry(key)
                .and_modify(|existing| match existing {
                    Value::Single(prev_val) => *existing = Value::Multiple(vec![prev_val, value]),
                    Value::Multiple(v) => v.push(value),
                })
                .or_insert(Value::Single(value));
        }
        QueryString { data }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_test() {
        let query_string = "a&a=1&&&a=&b=12&";
        let query_string = QueryString::from(query_string);
        let mut expected_data = HashMap::with_capacity(2);
        expected_data.insert("a", Value::Multiple(vec!["", "1", ""]));
        expected_data.insert("b", Value::Single("12"));
        assert_eq!(expected_data, query_string.data);
    }
}
