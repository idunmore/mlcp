use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::ffi::OsStr;

use indicatif::{ProgressBar, ProgressStyle};

use console::style;

use clap::{Parser};

// Significant File Types
const MUSIC_FILE_TYPES: [&str; 17] = [
    "aac", "aiff", "ape", "dff", "dsd", "dsf", "dxd", "flac", "iso", "m4a",
    "m4p", "mp3", "oga", "ogg", "wav", "wma", "wmv"
];

const AUDIO_FILE_TYPES: [&str; 28 ] = [
    "3gp", "aa", "aax", "act", "amr", "au", "awb", "dct", "dss", "dvf", "gsm", "iklax", "ivs",
    "m4b","mmf", "mpc","msv","mogg", "opus","ra","rm","raw","sln","tta", "vox","wmv","wv","webm"
];

// Alburm Art Filenames (Folder Level)
const ALBUM_ART_FILENAMES: [&str; 9] = ["album", "cover", "small_cover", "large_cover", "folder",
    "thumb", "albumartsmall", "albumartmedium", "albumartlarge"
];

// Album Art Extensions (Folder Level)
const ALBUM_ART_EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "png"];

// Operation Indicators
const PURGE: &str = "PURGED";
const BACKUP: &str = "BACKED-UP";
const SIMULATE: &str = "SIMULATED";
const ERROR: &str = "ERROR ";

// Path and File
const WILDCARD: &str = "**/*.*";
const NO_PATH: &str = "";
const NO_EXTENSION: &str = "";
const NO_FILE_NAME: &str = "";
const NO_CHAR: char = ' ';

// Resource fork characters 1 & 2 (i.e. "._")
const RES_FORK_1: char = '.';
const RES_FORK_2: char = '_';

// Exit Codes
const SUCCESS: i32 = 0;
const PATH_DOES_NOT_EXIST: i32 = 1;

#[derive(Parser, Debug)]
/// Music Library Crud Purge - Purge, or backup, "crud" files from a specified music library.
/// 
/// "Crud" files are any file that isn't one of the types designated to keep.
/// By default, this will delete any non-music file (see --list_types), but will
/// preserve folder-level album art.
/// 
/// Unless the --purge option is specified, NO changes to the library will occur!
/// This allows simulation of the purge/backup process to see what files will
/// be affected.
#[clap(author, version, about)]
struct Args {    
    /// Root folder for the music library to be purged.
    /// 
    /// All sub-folders will be processed recursively; specifying the root of
    /// the library will process all files in the library.  You can process a 
    /// single artist or album by specifying its respective path.
    #[clap(required=true, conflicts_with="list-types")]
    library_path: Option<String>,
    
    /// Root folder for backing up purged files.
    /// 
    /// If [BACKUP_PATH] is specified, files are moved here instead of deleted.
    /// The original folder structure is preserved, so they can be merged back
    /// into the library simply by copying the backup root to the library root.
    #[clap(conflicts_with="list-types")]
    backup_path: Option<String>,

    /// Perform the actual file purge
    /// 
    /// The "purge" flag must be specified to perform the actual purge
    /// operation.  Otherwise NO changes occur, and the process is just
    /// simulated so the affects can be evaluated (with -v | --verbose)
    /// prior to making them permanent.  
    #[clap(short, long, conflicts_with="list-types")]
    purge: bool,

    /// Delete folder-level album art.
    /// 
    /// Causes folder-level album art to be deleted; useful if space is at a
    /// premium (or when all files have embedded art and the folder-level files
    /// are holdovers from a download).
    #[clap(short, long, conflicts_with="list-types")]
    delete_art: bool,

    /// Keep other (non-music) audio files.
    /// 
    /// "Other" audio files are any audio file type that is not commonly used
    /// to store music.  By default, such files are DELETED (or backed up, if
    /// the -b | --backup flag is specified).
    #[clap(short, long, conflicts_with="list-types")]
    keep_other_audio: bool,

    /// List "music" vs. "audio" file types.
    /// 
    /// Lists both the "Music" files types, which are NEVER purged (green), as
    ///  well as "other Audio" file types - which are DELETED by default (red).
    #[clap(short, long)]
    list_types: bool,

    /// Enables verbose output.
    /// 
    /// Outputs the full path of every file or folder that is touched,
    /// along with the operation performed on it: PURGED (deleted), MOVED
    /// (backed-up), DIR (directory;not touched), RES (resource, skipped)
    #[clap(short, long, conflicts_with="list-types")]
    verbose: bool,
}

// Main entry point
fn main() {
    // Parse the command line ...  and take the appropriate action(s).
    let args = Args::parse();

    // List the Music and Audio File Types for the user's reference, then exit.
    if args.list_types { 
        list_types();
        exit(SUCCESS);
    }

    // From here, we are actually doing the mlcp tasks.

    // Does Library Path exist?  
    let library_path = args.library_path.unwrap_or(String::from(NO_PATH));
    if !Path::new(&library_path).exists() {
        eprintln!("Library path \"{}\" does not exist.", library_path );
        exit(PATH_DOES_NOT_EXIST);
    }    
        
    // Backups are enabled by specifying a BACKUP_PATH; is there one?
    let backup_enabled = match args.backup_path { None => false, Some(_) => true };
    let backup_root = args.backup_path.unwrap_or(String::from(NO_PATH));

    // If specified, the BACKUP_PATH must exist!
    if backup_enabled && !Path::new(&backup_root).exists() {
        eprintln!("Backup path \"{}\" does not exist.", backup_root );
        exit(PATH_DOES_NOT_EXIST);
    }

    // Get the files/directories for all items in the specified library_path
    let library_paths = get_library_paths(&library_path);

    // Build the PURGE file list ...
    let purge_file_list = 
        build_purge_file_list(library_paths, args.delete_art, args.keep_other_audio);   
    // ... and process the resultant files ...    
    
    // Option to wrap the progress bar, so we can optionally create it based
    // on verbose value ...
    let bar: Option<ProgressBar> =
        if !args.verbose { Some(ProgressBar::new(purge_file_list.len() as u64)) } else { None };
   
    if let Some(b) = &bar {
        b.set_style(ProgressStyle::default_bar()
            .template("{spinner} {bar:20.cyan/blue} {pos:>7}/{len:7} {msg:40!}"));
    }    
    
    // Get PathBuf instances for the two path strings ...
    let backup_dir = PathBuf::from(&backup_root);
    let source_dir = PathBuf::from(&library_path);
    // Error and Processed File Counts (can be different to number of files reported from glob)
    let mut err_count = 0;
    let mut proc_count = 0;
    // Which operation we're using.
    let op = 
        if args.purge && backup_enabled { BACKUP } else if args.purge { PURGE } else { SIMULATE };
    // ... and process all the files in the purge file list.
    for file in purge_file_list {
        proc_count += 1;       
        let msg = opt_osstr_to_string(file.file_name(), NO_FILE_NAME);
        // Process the file
        match purge_or_backup_file(&file, &source_dir, &backup_dir, backup_enabled, args.purge) {
            Ok(p) => { if args.verbose { println!("[{}] {}", op, file.display() ); } },
            Err(e) => {
                err_count += 1;
                print_verbose(
                    style(format!("[{}] {}", ERROR, e.display())).red().to_string(),
                    args.verbose
                );
                
            }
        }  
        
        // Only attempt to display/update the progress bar in non-verbose mode.
        if let Some(b) = &bar {
            b.set_message(msg);
            b.inc(1);
        }        
    }

    // Finish up the progress bar, if we are in non-verbose mode
    if let Some(b) = bar { b.finish(); }
    
    let exit_msg;
    if err_count == 0 { 
        exit_msg = format!("{} files successfuly {}.", proc_count, op );
    } else {
        exit_msg = 
            style(format!("{} errors out of {} files.", err_count, proc_count)).red().to_string();       
    }
    print_verbose(exit_msg, args.verbose);
    exit(err_count);
}

// Get the paths of all the files that are in the libary.
fn get_library_paths(library_path: &str) -> Vec<PathBuf> {
    // Build the appropriate glob path string  ...
    let glob_path = Path::new(library_path).join(WILDCARD);
    let glob_path_str = glob_path.to_str().expect("Invalid library_path.");
    // ... get the full list of files and directories therein ...
    let glob_paths = glob(&glob_path_str).expect("Glob request failed.");

    // Package the paths a vector for later processing.
    let mut lib_paths = Vec::<PathBuf>::new();
    for path in glob_paths { lib_paths.push(path.expect("Glob path error.")); }
    lib_paths
}

// Output the list of file types (extensions), for Music and Audio files.
fn list_types() {
   print_list("Music file types: ", &MUSIC_FILE_TYPES, true);
   print_list("Audio file types: ", &AUDIO_FILE_TYPES, false);  
}

// Build the list of Album Art files to keep.
fn build_keep_art_file_list(delete_art: bool) -> Vec<String> {
    let mut art_file_list = Vec::new();
    if !delete_art {
        for fname in ALBUM_ART_FILENAMES {
            for ext in ALBUM_ART_EXTENSIONS {
                art_file_list.push( format!("{}.{}", fname, ext ));
            }
        }
    }
    art_file_list
}

// Builds the potential list of file extensions that we will be keeping.
fn build_keep_extensions_list(keep_other_audio: bool) -> Vec<String> {
    let mut keep_extensions = Vec::new();
    // We always include the MUSIC file types.
    for ext in MUSIC_FILE_TYPES { keep_extensions.push(String::from(ext)) }
    // Add AUDIO file types, if we are keeping them.
    if keep_other_audio {
        for other_ext in AUDIO_FILE_TYPES {
            keep_extensions.push(String::from(other_ext));
        }
    }
    keep_extensions
}

// Get the list of extensions that we are intending to keep AND that ACTUALLY
// exist in the library.  
fn get_actual_extensions(library_paths: &Vec<PathBuf>, keep_other_audio: bool) -> Vec<String> {
    // Build the list of extensions we want to keep, if they exist.
    let keep_extensions = build_keep_extensions_list(keep_other_audio);

    // Now create a list of the extensions that ACTUALLY exist in the library.
    let mut extensions = Vec::new();
    for file in library_paths {        
        // Lossy string conversion is fine; these extensions are always UTF-8.
        let extension = opt_osstr_to_string(file.extension(), NO_EXTENSION).to_lowercase();
        // Add this extension only if it is part of the "keep" list, and is
        // NOT already in the extensions list.
        if !extensions.contains(&extension) && keep_extensions.contains(&extension) {
            extensions.push(extension);
        }                         
    }   
    extensions
}

// Builds the list of files to be purged.
fn build_purge_file_list(
    library_paths: Vec<PathBuf>,
    delete_art: bool,
    keep_other_audio: bool
) -> Vec<PathBuf> {
    let mut purge_file_list = Vec::new();

    // Get the list of art files and extensions we'll be keeping.
    let art_file_list = build_keep_art_file_list(delete_art);    
    // Get the list of actual extensions we will retain.
    let actual_extensions = get_actual_extensions( &library_paths, keep_other_audio);
    
    for file in library_paths {
        // Skip the file if it is a directory.
        if file.is_dir() { continue; }
        
        // Lossy conversion is fine; the part of the filename we're looking for
        // will always be UTF-8 (or won't be present).
        let file_name = opt_osstr_to_string(file.file_name(), NO_FILE_NAME);
         
        // Don't delete resource forks, as they'll auto-delete when their
        // parent file is removed.
         if is_resource_fork(&file_name) { continue; }

        // Is this file on the list of art files to be kept?
        if art_file_list.contains(&file_name.to_lowercase()) { continue; }
        
        // If it has an extension we're supposed to keep, keep it.        
        if actual_extensions.contains(
            &opt_osstr_to_string(file.extension(), NO_EXTENSION).to_lowercase()
        )
        {
            continue;
        }   

        purge_file_list.push(file); 
    }
    
    purge_file_list
}

// Determines if the file_name indicates a macOS resource fork (i.e. starts with "._").
fn is_resource_fork(file_name: &str) -> bool {
    // Must be at least 2 characters long to be a fork.
    if file_name.len() <2 { return false; }
    // Look at the characters individually, so as to prevent subscript issues
    // with multi-byte characters.
    if  file_name.chars().nth(0).unwrap_or(NO_CHAR) != RES_FORK_1 { return false; }
    if  file_name.chars().nth(1).unwrap_or(NO_CHAR) != RES_FORK_2 { return false; }
    true
}

// Purges, or moves (backs up) the specified file.
fn purge_or_backup_file(
    path: &PathBuf,
    library_path: &PathBuf,
    backup_path: &PathBuf,
    backup: bool,
    purge: bool
) -> Result<PathBuf, PathBuf>
{
    // If backup is enabled, backup the file first ...
    if backup && purge {
        if backup_file(&path, library_path, backup_path).is_err() {
            return Err(path.to_path_buf());
        }
    }
    // ... then purge the file as needed ...
    if purge {
        if fs::remove_file(&path).is_err() {
            eprintln!("Could not purge: {}", path.display());
            return Err(path.to_path_buf());
        }
    }
    Ok(path.to_path_buf())
}

// Backup the specified file, creating the target directory if needed.
fn backup_file(
    path: &PathBuf,
    library_path: &PathBuf,
    backup_path: &PathBuf
) -> Result<PathBuf, PathBuf>
{
    // Get the path to copy this file TO.         
    let relative_source_path = 
        path.strip_prefix(library_path).unwrap_or(Path::new(NO_PATH)).to_path_buf();
    let target_path = backup_path.join(relative_source_path);
            
    // Create the target directory IF needed ...
    let target_dir = target_path.parent().unwrap();
    if !target_dir.exists() { 
        if fs::create_dir_all(target_dir).is_err() {
            eprintln!("Could not create target directory: {}", target_dir.display());
            return Err(target_dir.to_path_buf()); }
    }
               
    // We use copy here, instead of "move", as "move" can only target the
    // same volume that the source files resides on.
    if fs::copy(&path, &target_path).is_ok() {
        return Ok(target_path);
    } else {
        eprintln!("Could not backup: {} -> {}", path.display(), target_path.display());
        return Err(target_path.to_path_buf());
    }    
}

// Prints File Type List
fn print_list( prefix: &str, arr: &[&str], keep: bool ) {
    // Set the output style (color): Default KEEP (GREEN); default DELETE (RED)
    let styled_prefix = if keep { style(prefix).green() } else { style(prefix).red() };
    let mut list: String = styled_prefix.to_string();
    
    // Add items to the list
    for ext in arr {
        list.push_str(ext);
        list.push_str(", ");        
    }

    // Remove the trailing comma and space.
    list.pop();
    list.pop();
    println!("{}", list);
}

// Prints specified text IF in VERBOSE mode.
fn print_verbose( text: String, verbose: bool) {
    if verbose { println!("{}", text); }
}

// Utility Functions

// Creates a (potentially lossy) string from Option<&OsStr>; for code readability.
fn opt_osstr_to_string(opt_osstr: Option<&OsStr>, default: &str) -> String {
    // Unwrap the OsStr from the Option, providing a default if it is "None", and then convert
    // the wrapped Cow<str> to a normal string, allowing for lossy conversion to UTF_8
    String::from(opt_osstr.unwrap_or(OsStr::new(default)).to_string_lossy())
}
