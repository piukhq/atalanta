use std::collections::HashMap;

type Data = HashMap<String, u32>;

trait Formatter {
    fn format(&self, data: &Data, buf: &mut String);
}

struct Output;

impl Output {
    fn generate<T: Formatter>(g: T, s: &mut String) {
        // backend operations...
        let mut data = HashMap::new();
        data.insert("one".to_string(), 1);
        data.insert("two".to_string(), 2);
        // generate report
        g.format(&data, s);
    }
}

struct Api;
impl Formatter for Api {
    fn format(&self, data: &Data, buf: &mut String) {
        for (k, v) in data {
            buf.push('[');
            for (k, v) in data.into_iter() {
                let entry = format!(r#"{{"{}":"{}"}}"#, k, v);
                buf.push_str(&entry);
                buf.push(',');
            }
            if !data.is_empty() {
                buf.pop(); // remove extra , at the end
            }
            buf.push(']');
        }
    }
}

struct Blob;
impl Formatter for Blob {
    fn format(&self, data: &Data, buf: &mut String) {
        for (k, v) in data {
            let entry = format!("{} {}\n", k, v);
            buf.push_str(&entry);
        }
    }
}

fn main() {
    let mut s = String::from("");
    Output::generate(Blob, &mut s);
    println!("{}", s);
    assert!(s.contains("one 1"));
    assert!(s.contains("two 2"));

    s.clear(); // reuse the same buffer
    Output::generate(Api, &mut s);
    println!("{}", s);
    assert!(s.contains(r#"{"one":"1"}"#));
    assert!(s.contains(r#"{"two":"2"}"#));
}
