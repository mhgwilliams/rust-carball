#[macro_use]
extern crate log;

use carball::analysis::CarballAnalyzer;
use carball::outputs::RangeChecker;
use carball::outputs::{
    DataFrameOutputFormat, DataFramesOutput, MetadataOutput, ParseOutputWriter,
};
use carball::CarballParser;
use simplelog::*;
use std::path::PathBuf;
use structopt::StructOpt;

use std::fs; // Hogan added code
use std::panic::{self, AssertUnwindSafe}; // handling errors parsing replays


#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, parse(from_os_str))]
    input_dir: PathBuf, // Rename input to input_dir
    #[structopt(short, parse(from_os_str))]
    output_dir: PathBuf,
    #[structopt(long)]
    skip_data_frames: bool,
    #[structopt(long)]
    skip_write_data_frames: bool,

    #[structopt(required_unless_one(&["skip_data_frames", "skip_write_data_frames"]), possible_values = &DataFrameOutputFormat::variants(), case_insensitive = true)]
    data_frame_output_format: Option<DataFrameOutputFormat>,

    #[structopt(long)]
    skip_checks: bool,

    #[structopt(long)]
    skip_analysis: bool,
}

fn process_file(input_path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn std::error::Error>> {
    // ... (Move the contents of the loop inside this function)

    let result = panic::catch_unwind(AssertUnwindSafe(|| -> Result<(), Box<dyn std::error::Error>> {
        // Hogan Code
        // Extract the first five characters of the input file name
        let file_stem = input_path.file_stem().unwrap().to_str().unwrap();
        let file_prefix = &file_stem[..5.min(file_stem.len())];

        let carball_parser = CarballParser::parse_file(input_path.clone(), true)?;

        let metadata =
            MetadataOutput::generate_from(&carball_parser.replay, &carball_parser.frame_parser);
        let data_frames = if opt.skip_data_frames {
            None
        } else {
            Some(
                DataFramesOutput::generate_from(&carball_parser.frame_parser)?,
            )
        };

        if !opt.skip_data_frames && !opt.skip_checks {
            if let Some(_data_frames) = &data_frames {
                let range_checker = RangeChecker::new();
                range_checker
                    .check_ranges(_data_frames)?;
            }
        }

        let parse_output_writer =
            ParseOutputWriter::new(opt.output_dir.clone(), opt.data_frame_output_format);
        if opt.skip_write_data_frames {
            parse_output_writer
                .write_outputs(Some(&metadata), None, file_prefix)?;
        } else {
            parse_output_writer
                .write_outputs(Some(&metadata), data_frames.as_ref(), file_prefix)?;
        }

        if !opt.skip_data_frames && !opt.skip_analysis {
            let analyzer = CarballAnalyzer::analyze(&carball_parser, &metadata, &data_frames.unwrap())?;
            analyzer.write(opt.output_dir.clone())?;
        }

        info!("Processed file: {:?}", input_path);

        // ... (Rest of the processing code, replacing 'expect' calls with '?')
        Ok(())
    }));

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Panic occurred while processing the file: {:?}", e).into()),
    }
}


fn main() {
    setup_logging();

    let opt = Opt::from_args();
    // dbg!(&opt);
    info!("{:?}", &opt);

    // Hogan Code - Check if the input directory exists
    if !opt.input_dir.exists() || !opt.input_dir.is_dir() {
        panic!("Input directory does not exist or is not a directory.");
    }

    // Hogan Code - Check if the output directory exists, and create it if it doesn't
    if !opt.output_dir.exists() {
        fs::create_dir_all(&opt.output_dir).expect("Failed to create output directory.");
    }

    // List all files in the input directory
    let input_files = fs::read_dir(&opt.input_dir).expect("Failed to read input directory.");

    // Iterate through the input files and process each one
    for entry in input_files {
        let entry = entry.expect("Failed to read entry in input directory.");
        let input_path = entry.path();

        // Make sure the entry is a file
        if !input_path.is_file() {
            continue;
        }

        // Process the file and handle errors
        match process_file(&input_path, &opt) {
            Ok(_) => {
                info!("Processed file: {:?}", input_path);
            }
            Err(e) => {
                error!("Failed to process file {:?}, error: {:?}", input_path, e);
            }
        }

    }

    info!("fin");

}

fn setup_logging() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    // println!("Testing logging");
    // debug!("debug");
    // info!("info");
    // warn!("warn");
    // error!("error");
}
