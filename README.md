# mlcp
## Music Library "Crud" Purge (mlcp) - Purges non-music files ('crud') from music libraries.

A command-line utility for easily removing extraneous "**crud**" files, that tend to accumulate in music libraries.

This is useful for keeping libraries focused and "tidy", as well as for removing unnecessary files to minimize wasted space and maximize music capacity on devices such as DAPs (Digital Audio Players) and other limited-storage devices.

---

**WARNING:** *By design, this tool **deletes files**.  Use at your own risk, proceed with caution, and only operate against libraries for which you have a known-good back-up.*  It is highly recommended to run a test, **omitting** the "-p" or "--purge" options, and using "-v" or "--verbose" to see which files are affected, before running with the "-p" or "--purge" option enabled.

---

## What constitutes "crud"?

A file is considered to be "**crud**" if it is not one of the designaged file-types to keep.  By default, common music file types and folder-level album art is retained and **ALL** other files are considered to be "**crud**".  "Crud" files are either purged (deleted), or if [BACKUP_PATH] is specified they will be backed-up (with a matching folder structure) prior to deletion.

Options are provided to:

* Remove folder-level album art.
* Keep documentation and booklets (.txt and .pdf files).
* Keep other *non-music* audio files.

To see what file types are considered to be *music* files, vs. *other audio* and *documentation*, use the "-l" or "list-types" option:

<pre><code>mlcp -l</code></pre>

This will yield output in the form:

**Music file types:** aac, aiff, ape, dff, dsd, dsf, dxd, flac, iso, m4a, m4p, mp3, oga, ogg, wav, wma, wmv

**Audio file types:** 3gp, aa, aax, act, amr, au, awb, dct, dss, dvf, gsm, iklax, ivs, m4b, mmf, mpc, msv, mogg, opus, ra, rm, raw, sln, tta, vox, wmv, wv, webm

**Document/booklet file types:** txt, pdf

---

## Usage:

<pre><code>mlcp 0.3.0
Ian Dunmore
Music Library Crud Purge - Purges non-music files ('crud') from music libraries.

USAGE:
    mlcp [OPTIONS] &ltLIBRARY_PATH&gt [BACKUP_PATH]

ARGS:
    &ltLIBRARY_PATH&gt    Root folder for the music library to be purged
    &ltBACKUP_PATH&gt     Root folder for backing up purged files

OPTIONS:
    -a, --art            Purge folder-level album art
    -d, --documents      Keep document/booklet files (e.g. .txt, .pdf)
    -h, --help           Print help information
    -l, --list-types     List "music" vs. "audio" file types
    -o, --other-audio    Keep other (non-music) audio files
    -p, --purge          Perform the actual file purge
    -v, --verbose        Enables verbose output
    -V, --version        Print version information</code></pre>

**NOTES:** 

 * The "-p" or "--purge" option *must* be specified to make actual changes to the files/library - otherwise operations are only simulated (to allow verification of which files will be affected).
 
 * The "--help" option will provide more detailed help information than just using "-h".

 * If [BACKUP_PATH] is specified, then "crud" files will be backed-up to that location, using the same folder structure as the <LIBRARY_PATH>, prior to being purged from the library.  If a file cannot be backed-up, it will **not** be deleted from the library.

 * If a path contains folders with spaces in the names, place qoutes around the path name (e.g., "~/Users/jsmith/My Music Library").

<br>

### Usage Examples:

<br>

List all the files that would be purged, by default, for the music library located in the "~/users/jsmith/music" folder:

<pre><code>mlcp ~/users/jsmith/music -v</code></pre>

Purge all default "crud" files, for the music library located in the "~/users/jsmith/music" folder:

<pre><code>mlcp ~/users/jsmith/music -p</code></pre>

Backup all "crud" files to the folder "//Volumes/Backup/music" for the library located in "~/users/jsmith/music":

<pre><code>mlcp ~/users/jsmith/music //Volumes/Backup/music --purge</code></pre>

The same as the above, but *keeping* documentation/booklets in the library, and listing every file that is backed-up:

<pre><code>mlcp ~/users/jsmith/music //Volumes/Backup/music -p -d -v</code></pre>

To remove the maximum amount of non-music "crud" files, without backing them up, for the library located at: "~/users/jsmith/music".  Note that this will remove all folder-level album art, so art will not be displayed by your player software or device unless it is embedded in the individual music files.

<pre><code>mlcp ~/users/jsmith/music --art</code></pre>
