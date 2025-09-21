const ALPH: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+\\=-";

fn encode(binary: &[u8], len: usize) -> String {
    let mut res = String::new();
    let byte_alph = ALPH.as_bytes();
    // let len1 = len / 3;
    let len1 = len % 3;
    // let len = len - len1;
    for i in (0..(len - len1)).step_by(3) {
        let x1 = binary[i];
        let x2 = binary[i + 1];
        let x3 = binary[i + 2];

        res.push(byte_alph[(x1 >> 2) as usize] as char);
        res.push(byte_alph[(((x1 & 3) << 4) ^ (x2 >> 4)) as usize] as char);
        res.push(byte_alph[(((x2 & 15) << 2) ^ (x3 >> 6)) as usize] as char);
        res.push(byte_alph[(x3 & 63) as usize] as char);
    }

    match len1 {
        1 => {
            let x1 = binary[len - 1];
            res.push(byte_alph[(x1 >> 2) as usize] as char);
            res.push(byte_alph[((x1 & 3) << 4) as usize] as char);
            res.push('=');
            res.push('=');
        }
        2 => {
            let x1 = binary[len - 2];
            let x2 = binary[len - 1];
            res.push(byte_alph[(x1 >> 2) as usize] as char);
            res.push(byte_alph[(((x1 & 3) << 4) | (x2 >> 4)) as usize] as char);
            res.push(byte_alph[((x2 & 15) << 2) as usize] as char);
            res.push('=');
        }
        _ => {}
    }

    res
}

enum Error64 {
    Symbol((usize, usize, char)),
    Lenght((usize, usize)),
    Padding((usize, usize)),
    Uknown(String),
    Ok(Vec<u8>),
}

fn error_handling(y: u8, letter: usize, row: usize, base: &[u8]) -> Result<u8, Error64> {
    match y {
        0..=64 => Ok(y),
        // 64 => Err(Error64::Padding((row, letter + 1))),
        65.. => Err(Error64::Symbol((row, letter + 1, base[letter] as char))),
    }
}

fn decode(filename: &str) -> Error64 {
    let mut res: Vec<u8> = Vec::new();
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => return Error64::Uknown(e.to_string()),
    };

    let reader = BufReader::new(file);

    let rows: Vec<String> = match reader.lines().collect::<Result<_, _>>() {
        Ok(rows) => rows,
        Err(e) => return Error64::Uknown(e.to_string()),
    };
    let byte_alph = ALPH.as_bytes();
    let mut k = 1;
    let count_rows = rows.len();
    let mut flag_EOC = false;
    for base in rows {
        let base = base.as_bytes();
        let len = base.len();
        if base[0] == '-' as u8 {
            k += 1;
            continue;
        } else if flag_EOC { println!("Наявні дані після кінця повідомлення");
            return  Error64::Ok(res);
        }


        let base_len = base.len();

        if base[base_len - 1] != '=' as u8 && base[base_len - 2] != '=' as u8 {
            if (base.len() != 76) && count_rows != k {
                return Error64::Lenght((k, base.len()));
            }
        }

        if base.len() % 4 != 0 {
            return Error64::Lenght((k, len));
        }
        for i in (0..len).step_by(4) {
            let y1: u8 = byte_alph.iter().position(|&x| x == base[i]).unwrap_or(100) as u8;
            match error_handling(y1, i, k, base) {
                Err(e) => return e,
                _ => {}
            }

            let mut y2: u8 = byte_alph
                .iter()
                .position(|&x| x == base[i + 1])
                .unwrap_or(100) as u8;
            match error_handling(y2, i + 1, k, base) {
                Err(e) => return e,
                _ => {}
            }
            if y1 == 64 {
                return Error64::Padding((k, i + 1));
            }
            if y2 == 64 {
                return Error64::Padding((k, i + 2));
            }
            let y3: u8 = byte_alph
                .iter()
                .position(|&x| x == base[i + 2])
                .unwrap_or(100) as u8;
            match error_handling(y3, i + 2, k, base) {
                Err(e) => {
                    return e;
                }
                _ => {}
            }

            let y4: u8 = byte_alph
                .iter()
                .position(|&x| x == base[i + 3])
                .unwrap_or(100) as u8;
            match error_handling(y4, i + 3, k, base) {
                Err(E) => {
                    return E;
                }
                _ => {}
            }

            res.push(y1 << 2 | (y2 >> 4));
            if i + 4 == len && y3 == 64 {
                // println!("{k}, {count_rows}");
                if y4 == 64 {
                    flag_EOC = true;
                    // return Error64::Ok(res);
                } else {
                    return Error64::Padding((k, i + 1));
                }
            }
            if !flag_EOC {
            res.push((y2 << 4) | (y3 >> 2));}
            if i + 4 == len && y4 == 64 {
                flag_EOC = true;
                // return Error64::Ok(res);
            }
            if !flag_EOC {
            res.push(((y3 & 3) << 6 | y4));}
        }

        k += 1;
    }

    Error64::Ok(res)
}

use clap::{Parser, Subcommand};
use std::io::{self, BufRead, BufReader};

#[derive(Subcommand)]
enum Commands {
    Encode {
        input_file: String,
        output_file: Option<String>,
    },
    Decode {
        input_file: String,
        output_file: Option<String>,
    },
}

use std::fs::{self, File};
use std::io::Write;

fn read_file(filename: &str) -> io::Result<Vec<u8>> {
    let bytes = fs::read(filename)?;
    Ok(bytes)
}
fn write_bytes(filename: &str, data: &[u8]) -> io::Result<()> {
    fs::write(filename, data)?;
    Ok(())
}

fn write_base64(filename: &str, data: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all("--- Begin Base64 code ---\n".as_bytes())?;
    for line in data.as_bytes().chunks(76) {
        file.write_all(line)?;
        file.write_all(b"\n")?;
    }

    file.write_all("--- End Base64 code ---\n".as_bytes())?;
    Ok(())
}

#[derive(Parser)]
// #[command(name = "base64cli")]
// #[command(about = "CLI для encode/decode файлів у Base64", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Encode {
            mut input_file,
            output_file,
        } => {
            // println!("{:?}, {:?}", input_file, output_file);
            let a = read_file(input_file.as_str());
            // println!("{:?}",a.unwrap());
            match read_file(input_file.as_str()) {
                Ok(to_encode) => {
                    let encoded = encode(&to_encode, to_encode.len());
                    // println!("Encoded: {:?}", to_encode);
                    // println!("Encoded: {}", encoded);
                    match output_file {
                        Some(name_out) => {
                            write_base64(&name_out, &encoded).unwrap();
                        }
                        None => {
                            let new_name =
                                input_file.split(".").next().unwrap().to_string() + ".base64";
                            write_base64(&new_name, &encoded).unwrap();
                        }
                    }
                }
                Err(e) => println!("{:?}", e.to_string()),
            }
        }
        Commands::Decode {
            input_file,
            output_file,
        } => {
            // println!("Input: {:?}", input_file);
            match decode(&input_file) {
                Error64::Ok(res) => match output_file {
                    Some(name_out) =>
                        {
                             // println!("{:?}",res);
                            write_bytes(&name_out, &res).unwrap()},
                    None => {
                         // println!("{:?}", res);
                        let new_name = input_file.split(".").next().unwrap().to_string() + ".bin";
                        write_bytes(&new_name, &res).unwrap();
                    }
                },
                Error64::Symbol((row, sym, ch)) => {
                    println!(
                        "Рядок {}, символ {}, Некоректний вхідний символ ({})",
                        row, sym, ch
                    )
                }
                Error64::Padding((row, sym)) => {
                    println!(
                        "Рядок {}, символ {}: Неправильне використання падіннгу ",
                        row, sym
                    )
                }
                Error64::Lenght((row, l)) => {
                    println!("Рядок {row}: Некоректна довжина рядку ({l}) ");
                }
                Error64::Uknown(e) => {
                    println!("{:?}", e);
                }
            }
            // println!("{:?}, {:?}", input_file, output_file);
        }
    }
}
