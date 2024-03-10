use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("yo1956")
        .about("Rust head")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("The files to print")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
                .help("Number of lines to print")
                .default_value("10"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .help("Number of bytes to print")
                .takes_value(true)
                .conflicts_with("lines"),
        )
        .get_matches();

    let lines = matches
        .value_of("lines")
        .map(parse_positive_int) // OptionがSomeの場合にのみ関数を適用し、Noneの場合は何もしない
        .transpose() //Option<Result>をResult<Option>に変換する
        .map_err(|e| format!("illegal line count -- {}", e))?;

    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(), // filesは少なくとも1つの値を持っているはずなので、unwrapしても問題ない
        lines: lines.unwrap(),
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let num_files = config.files.len();

    for (file_num, filename) in config.files.iter().enumerate() {
        match open(&filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(mut file) => {
                if num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }

                if let Some(num_bytes) = config.bytes {
                    /* 指定されたバイト数ファイルから読み出し表示する */
                    let mut handle = file.take(num_bytes as u64); // fileからnum_bytesバイト文だけ取り出すためのTakeストリームを作成
                    let mut buffer = vec![0; num_bytes]; // 0で初期化したnum_bytes長の可変なバッファ
                    let bytes_read = handle.read(&mut buffer)?;
                    print!(
                        "{}",
                        String::from_utf8_lossy(&buffer[..bytes_read]) //..bytes_read: バッファの先頭からbytes_readバイトまでのスライスを表す
                    );
                } else {
                    /* 指定された行数ファイルから読み出し表示する */
                    let mut line = String::new(); // 可変な文字列バッファをヒープ上に確保しスタックにlineを割り当て
                    for _ in 0..config.lines {
                        let bytes = file.read_line(&mut line)?;
                        if bytes == 0 {
                            break;
                        }
                        print! {"{}", line};
                        line.clear();
                    }
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse::<usize>() {
        Ok(num) if num > 0 => Ok(num),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    // 3は正の整数なのでOK
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // 数字でない文字列の場合はエラー
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // 0の場合もエラー
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}
