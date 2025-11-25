use std::env;
use std::fs;
use std::path::Path;

use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use sha1::{Digest, Sha1}; 
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};


fn read_object_raw(sha: &str) -> Vec<u8> {
    let (dir, file) = sha.split_at(2);
    let path = format!(".git/objects/{}/{}", dir, file);
    let compressed = fs::read(&path).expect("objet introuvable");
    let mut dec = ZlibDecoder::new(&compressed[..]);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).unwrap();
    out
}

fn split_header_body(raw: &[u8]) -> (&[u8], &[u8]) {
    let i = raw.iter().position(|&b| b == 0).expect("format objet invalide (pas de NUL)");
    (&raw[..i], &raw[i + 1..])
}

fn write_object(kind: &str, payload: &[u8]) -> String {
    let mut buf = format!("{} {}\0", kind, payload.len()).into_bytes();
    buf.extend_from_slice(payload);

    let mut h = Sha1::new();
    h.update(&buf);
    let sha = format!("{:x}", h.finalize());

    let (dir, file) = sha.split_at(2);
    let obj_dir = format!(".git/objects/{}", dir);
    let obj_path = format!("{}/{}", obj_dir, file);
    fs::create_dir_all(&obj_dir).unwrap();

    if !Path::new(&obj_path).exists() {
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&buf).unwrap();
        let compressed = enc.finish().unwrap();
        fs::write(&obj_path, compressed).unwrap();
    }
    sha
}
fn hex_to_bin(h: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(20);
    for i in (0..h.len()).step_by(2) {
        out.push(u8::from_str_radix(&h[i..i+2], 16).unwrap());
    }
    out
}


fn is_ignored(name: &str) -> bool {
    name == ".git" || name == "." || name == ".."
}

fn write_blob_from_file(path: &Path) -> String {
    let data = fs::read(path).unwrap();
    write_object("blob", &data)
}

fn write_tree_rec(dir: &Path) -> String {
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut buf = Vec::new();

    for ent in entries {
        let name_os = ent.file_name();
        let name = name_os.to_string_lossy().to_string();
        if is_ignored(&name) { continue; }

        let path = ent.path();
        let md = ent.metadata().unwrap();

        if md.is_dir() {
            let sha = write_tree_rec(&path);
            buf.extend_from_slice(b"40000");
            buf.push(b' ');
            buf.extend_from_slice(name.as_bytes());
            buf.push(0);
            buf.extend_from_slice(&hex_to_bin(&sha));
        } else {
            let sha = write_blob_from_file(&path);
            buf.extend_from_slice(b"100644");
            buf.push(b' ');
            buf.extend_from_slice(name.as_bytes());
            buf.push(0);
            buf.extend_from_slice(&hex_to_bin(&sha));
        }
    }

    write_object("tree", &buf)
}


fn write_commit(tree_sha: &str, parent: Option<&str>, message: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let timestamp = now.to_string();

    let author = "nathan-dnx <nathan.denoux@gmail.com.com>";

    let mut payload = String::new();
    payload.push_str(&format!("tree {}\n", tree_sha));

    if let Some(p) = parent {
        payload.push_str(&format!("parent {}\n", p));
    }

    payload.push_str(&format!("author {} {} +0000\n", author, timestamp));
    payload.push_str(&format!("committer {} {} +0000\n", author, timestamp));
    payload.push_str("\n");
    payload.push_str(message);
    payload.push_str("\n");

    write_object("commit", payload.as_bytes())
}


fn main() {
    eprintln!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: <program> <command>");
        return;
    }

    match args[1].as_str() {
        
        
        "init" => {
            if Path::new(".git").exists() {
                println!("Reinitialized existing Git repository");
            } else {
                fs::create_dir(".git").unwrap();
                fs::create_dir(".git/objects").unwrap();
                fs::create_dir(".git/refs").unwrap();
                fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
                println!("Initialized git directory");
            }
        }

        "cat-file" => {
            if args.len() < 4 || args[2] != "-p" {
                eprintln!("Usage: <program> cat-file -p <object_hash>");
                return;
            }

            let hash = &args[3];
            let (dir, file) = hash.split_at(2);
            let object_path = format!(".git/objects/{}/{}", dir, file);

            let compressed = fs::read(&object_path).expect("Object not found");
            let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();

            if let Some(null_index) = decompressed.iter().position(|&b| b == 0) {
                let content = &decompressed[null_index + 1..];
                print!("{}", String::from_utf8_lossy(content));
            } else {
                eprintln!("Invalid blob format");
            }
        }

        "hash-object" => {
            if args.len() < 3 {
                eprintln!("Usage: <program> hash-object [-w] <file>");
                return;
            }

            let mut write_flag = false;
            let mut file_arg_index = 2;

            if args[2] == "-w" {
                write_flag = true;
                file_arg_index = 3;
            }

            let file_path = &args[file_arg_index];
            let content = fs::read(file_path).expect("Unable to read file");

            
            let header = format!("blob {}\0", content.len());
            let mut store = Vec::new();
            store.extend_from_slice(header.as_bytes());
            store.extend_from_slice(&content);

            
            let mut hasher = Sha1::new();
            hasher.update(&store);
            let hash_bytes = hasher.finalize();
            let hash_str = format!("{:x}", hash_bytes);

            if write_flag {
                let (dir, file) = hash_str.split_at(2);
                let object_dir = format!(".git/objects/{}", dir);
                fs::create_dir_all(&object_dir).unwrap();

                let object_path = format!("{}/{}", object_dir, file);

                if !Path::new(&object_path).exists() {
                    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(&store).unwrap();
                    let compressed = encoder.finish().unwrap();
                    fs::write(&object_path, compressed).unwrap();
                }
            }

            println!("{}", hash_str);
        }

        
        "ls-tree" => {
            if args.len() < 4 || args[2] != "--name-only" {
                eprintln!("Usage: <program> ls-tree --name-only <tree_sha>");
                return;
            }

            let sha = &args[3];
            let raw = read_object_raw(sha);
            let (hdr, body) = split_header_body(&raw);

            let typ = std::str::from_utf8(hdr).unwrap().split_once(' ').unwrap().0;
            if typ != "tree" {
                eprintln!("objet non-tree");
                return;
            }

            let mut i = 0usize;
            while i < body.len() {
                while body[i] != b' ' { i += 1; } 
                i += 1;

                let name_start = i;
                while body[i] != 0 { i += 1; } 
                let name = std::str::from_utf8(&body[name_start..i]).unwrap();
                i += 1; 

                i += 20; 

                println!("{}", name);
            }
        }

        "write-tree" => {
            let sha = write_tree_rec(Path::new("."));
         println!("{}", sha);
        }   
       
        

        "commit-tree" => {
            if args.len() < 6 {
              eprintln!("Usage: <program> commit-tree <tree_sha> -p <parent_sha> -m <message>");
              return;
            }

         let tree_sha = &args[2];

         let mut parent: Option<String> = None;
          let mut message_start: Option<usize> = None;

         let mut i = 3;
         while i < args.len() {
              match args[i].as_str() {
                 "-p" => {
                      if i + 1 >= args.len() {
                         eprintln!("Missing value after -p");
                         return;
                        }  
                      parent = Some(args[i + 1].clone());
                     i += 2;
                    }
                  "-m" => {
                      message_start = Some(i + 1);
                     break;
                    }
                    _ => {
                      i += 1;
                    }
                }       
         }

         let message_start = match message_start {
              Some(idx) => idx,
             None => {
                 eprintln!("Missing -m <message>");
                 return;
            }
           };

         let parent_sha = match parent {
             Some(p) => p,
              None => {
                  eprintln!("Missing -p <parent_sha>");
                 return;
            }
         };

         let message = args[message_start..].join(" ");

          let commit_sha = write_commit(tree_sha, Some(&parent_sha), &message);
         println!("{}", commit_sha);
        }





    _ => {
            println!("unknown command: {}", args[1]);
        }















    }
}