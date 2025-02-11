use std::fs::{read, read_to_string, write};
use std::path::PathBuf;

use colorful::Colorful;
use structopt::StructOpt;

use chipo::{
    compile,
    error::{ChipoError, Result},
    reverse_parse,
};

use chipo_native::run;

#[derive(StructOpt)]
struct Opt {
    #[structopt(long, short)]
    file: PathBuf,

    #[structopt(long, short)]
    no_run: bool,

    #[structopt(long, short)]
    out_file: Option<PathBuf>,
}

fn read_from_file(file: &PathBuf) -> Result<Vec<u8>> {
    match file.extension().and_then(std::ffi::OsStr::to_str) {
        Some("s") => {
            let asm = read_to_string(file)?;
            compile(&asm)
        }
        Some("c8") | Some("ch8") => read(file).map_err(ChipoError::IOError),
        _ => Err(ChipoError::InvalidFile(file.to_str().unwrap().to_string())),
    }
}

fn write_to_file(file: &PathBuf, tokens: &[u8]) -> Result<()> {
    match file.extension().and_then(std::ffi::OsStr::to_str) {
        Some("s") => {
            let instructions = reverse_parse(tokens)?;
            write(file, instructions)?;
        }
        Some("c8") | Some("ch8") => {
            write(file, tokens)?;
        }
        _ => {
            return Err(ChipoError::InvalidFile(file.to_str().unwrap().to_string()));
        }
    }
    Ok(())
}

fn try_main(args: &Opt) -> Result<()> {
    let file = &args.file;
    let tokens = read_from_file(file)?;
    if let Some(out_path) = &args.out_file {
        write_to_file(out_path, &tokens)?;
    }

    if !args.no_run {
        run(&tokens)?;
    }

    Ok(())
}

fn main() {
    let args = Opt::from_args();
    match try_main(&args) {
        Ok(()) => {
            println!("chipo exited successfully!");
        }
        Err(err) => {
            eprintln!("{}", format!("error: {}", err).red());
        }
    }
}
