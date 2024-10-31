use serde_json;
use core::panic;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    match encoded_value.chars().next() {
        Some('i') => {
            if let Some((n,rest)) = 
            encoded_value.split_at(1).1
            .split_once('e').and_then(|(digits,rest)| {
                let n = digits.parse::<i64>().ok()?;
                Some((n, rest))
            })
            {
                return (n.into(), rest);
            }
        }
        Some('l') => {
            let mut values = Vec::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with("e"){
                let (v, remainder) = decode_bencoded_value(rest);
                values.push(v);
                rest = remainder;
            }
            return (values.into(), &rest[1..]);
    }
    Some('0'..='9') => {
        if let Some((len, rest)) = encoded_value.split_once(":") {
            if let Ok(len) = len.parse::<usize>() {
                return (rest[..len].to_string().into(), &rest[len..]);
                }
            }
        }
    _=>{}
    }
    panic!("Unhandled encoded value: {encoded_value}");
}

   /* if encoded_value.chars().next().unwrap().is_digit(10) {
        let colon_index = encoded_value.find(':').unwrap();
        let string = &encoded_value[colon_index + 1..];
        return serde_json::Value::String(string.to_string());
    } 
    else if encoded_value.chars().next().unwrap().is_alphabetic() 
    {
        let first = encoded_value.chars().nth(0).unwrap();
        let len = encoded_value.len();
        let mut list = Vec::new();


        if first == 'i'{
        let num_string = &encoded_value[1..len-1];
        let number = num_string.parse::<i64>().unwrap();
        serde_json::Value::Number(number.into())
        }
        else  {
            if encoded_value.chars().nth(1).unwrap() == 'e'{
                serde_json::Value::Array(list)
            }
            else {
            let i_index = encoded_value.find('i').unwrap();
            let s1 = decode_bencoded_value(&encoded_value[1..i_index]);
            let s2 = decode_bencoded_value(&encoded_value[i_index..len-1]);
            list.push(s1);
            list.push(s2);
            serde_json::Value::Array(list)
            }
        }
    }
    else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}*/

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
       // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.0.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
