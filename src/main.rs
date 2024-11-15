use anyhow::Context;
use serde_bencode;
use serde_json;
use serde::Deserialize;
use std::path::PathBuf;
use hashes::Hashes;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args{
    #[command(subcommand)] //I dont understand this either
    command: Command,
}


#[derive(Subcommand, Debug)]
enum Command {
    Decode { value: String },
    Info{ torrent: PathBuf },
}


/// A Metainfo file is also known as .torrent file
#[derive(Debug, Clone, Deserialize)]
struct Torrent {
    announce: String,
 
    info: Info,
}


#[derive(Debug, Clone, Deserialize)]
struct Info{
    /// The suggested name to store the file (or directory)
    name: String,
    
    /// The number of bytes in each piece the file is split into.
    /// 
    /// For the purposes of transfer, files are split into fixed-sized pieces which are all the same length
    /// except for possibly the last one which may be truncated. piece length is almost always a power of two, 
    /// most commonly 2^18 = 256k (BitTorrent prior to version 3.2 uses 2^20 =  1M as default).
    #[serde(rename = "piece length")]
    plength: usize,
    
    /// each entry of 'pieces' is the SHA1 hash of the piece at the corresponding index. 
    pieces: Hashes,
    
    #[serde(flatten)]
    keys: Keys,
}

/// There is a key length or a key files, but not both or neither.  
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Keys{
    /// If length is present then the dowmload represents a single file
    SingleFile { 
        /// the length of the file in bytes.
        length: usize 
    },
    /// Otherwise it represents a set of files which go in a directory structure.
    ///
    /// For the purpose of the other keys in 'Info', the multi-file case is treated as only having a single file,
    /// by concatenating the files in the order they appear in the files list.
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    /// The length for this file, in bytes.
    length: usize,
    
    /// Subsidiary name for this file, the last of which is actual file name. (a zero length list in an error case)
    path: Vec<String>,
}

/// serde_bencode to serde_json::Value is cooked therefore we had to bring back our og parser
#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    match encoded_value.chars().next() {
        Some('i') => {
            if let Some((n,rest)) = 
            encoded_value.split_at(1).1
            .split_once('e').and_then(|(digits,rest)| {
                let n = digits.parse::<i64>().ok()?;
                Some((n, rest)) }){
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
        Some('d') => {
            let mut dict = serde_json::Map::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with("e"){
                let (k, remainder) = decode_bencoded_value(rest);
                let k = match k {
                    serde_json::Value::String(k) => k,
                    _ => panic!("Key must be a string"),

                };
                let (v, remainder) =decode_bencoded_value(remainder);
                dict.insert(k, v);
                rest = remainder;
            }
            return (dict.into(), &rest[1..]);
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

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()>{
    let args = Args::parse();
    match args.command {
        Command::Decode { value} => {
            //Since the serde_bencode was failing previous test cases, we brought back the original parser to help parse it 
            let v = decode_bencoded_value(&value).0; 
            println!("{v}");
        }
        Command::Info { torrent } => {
            let dot_torrent = std::fs::read(torrent).context("read torrent fole")?;
            let t: Torrent = serde_bencode::from_bytes(&dot_torrent).context("decode torrent")?;
            eprintln!("{t:?}");
            println!("Tracker URL: {}", t.announce);
            if let Keys::SingleFile { length } = t.info.keys{
                println!("Length: {length}");
            } else {
                todo!()
            }
        }
    }
    
    Ok(())

    
}

mod hashes {
use serde::de::{self, Visitor, Deserialize, Deserializer};
use core::fmt;
    

#[derive(Debug, Clone)]
pub struct Hashes(pub Vec<[u8; 20]>);
#[derive(Debug, Clone)]
pub struct HashesVisitor;
    
impl<'de> Visitor<'de> for HashesVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a list of 20-byte hashes")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error, {
        if v.len() % 20 != 0 {
            return Err(E::custom(format!("invalid length {}", v.len())));
        }
        // TODO: use array_chunks when stable
        Ok(Hashes(
            v.chunks_exact(20)
            .map(|slice_20| slice_20.try_into().expect("Guranteed to be of length 20"))
            .collect(),
        ))
    }
}

impl<'de> Deserialize<'de> for Hashes{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de> 
        {
            deserializer.deserialize_bytes(HashesVisitor)
        }
}

}

//Self made parser2.0


// parser 1.0
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