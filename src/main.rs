#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

extern crate docker_image;
extern crate regex;
extern crate serde;
extern crate serde_json;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use clap::{Arg, App};
use regex::Regex;

lazy_static! {
    static ref INVALID_FILE_CHARAS: Regex = Regex::new("[^A-Za-z0-9._-]").unwrap();
}

/// An "Image" in our manifest
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
struct Image<'a> {
    image: &'a str,
    repository: &'a str,
    tag: &'a str,
    tarball: Cow<'a, str>,
}

impl<'a> Image<'a> {
    // Parse a Docker image name into an `Image` struct
    pub fn parse_image(image: &'a str) -> Self {
        let image_parts = docker_image::parse(image);
        Self {
            image: image,
            repository: image_parts.repository,
            tag: image_parts.tag.unwrap_or_else(|| "latest"),
            // TODO: More options
            tarball: Cow::from(format!(
                "images/{}.tar.gz",
                INVALID_FILE_CHARAS.replace_all(image, "_")
            )),
        }
    }
}

/// The Manifest is a dictionary of namespaces as keys and list of images as values
type Manifest<'a> = HashMap<&'a str, Vec<Image<'a>>>;

fn main() {
    let args = make_parser().get_matches();

    // Argument values
    let merge = args.occurrences_of("merge") > 0;
    let merge_from = args.value_of("merge-from").unwrap_or_else(|| "-");
    let namespace = args.value_of("namespace").unwrap(); // safe to unwrap because this is required
    let output_path = args.value_of("output").unwrap_or_else(|| "-");
    // safe to unwrap because this is required
    let images: Vec<&str> = args.values_of("image").unwrap().collect();

    match process(images.as_slice(), namespace, output_path, merge, merge_from) {
        Ok(()) => {}
        Err(e) => panic!("{}", e),
    };
}

/// "Inner" main that takes in arguments for testability
fn process<'a>(
    images: &'a [&'a str],
    namespace: &'a str,
    output_path: &'a str,
    merge: bool,
    merge_from: &'a str,
) -> Result<(), String> {
    // Parse the images and map them into `Image`
    let images: Vec<_> = images.into_iter().map(|s| Image::parse_image(s)).collect();

    let merge_input = if merge {
        let merge_reader = input_reader(merge_from)?;
        let merge_input = read(merge_reader)?;
        Some(merge_input)
    } else {
        None
    };

    let merge_manifest: Option<Manifest> = match merge_input {
        Some(ref merge_input) => {
            serde_json::from_str(merge_input).map_err(|e| {
                format!("Error deserializing merge input: {}", e)
            })?
        }
        None => None,
    };

    let mut manifest = Manifest::new();
    manifest.insert(namespace, images);

    if let Some(merge_manifest) = merge_manifest {
        for (namespace, images) in merge_manifest {
            manifest.insert(namespace, images);
        }
    }

    // Get writer
    let writer = output_writer(output_path)?;
    output(&manifest, writer)
}

/// Make a command line parser for options
fn make_parser<'a, 'b>() -> App<'a, 'b>
where
    'a: 'b,
{
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(
            "Generate a serialized manifest of Docker Image names, \
             components and additonal values from a list of Docker Images

             Each invocation of the CLI \
             allows one single namespace. You can pipe this to a subsequent invocation \
             of the CLI tool with the `--merge` flag to ask the CLI to merge the output \
             with whatever is fed into it from either the STDIN or the path specified by \
             --merge-from.

             If no `--output` is specified, the CLI will write to STDOUT.",
        )
        .arg(
            Arg::with_name("namespace")
                .long("namespace")
                .short("n")
                .takes_value(true)
                .required(true)
                .help(
                    "The namespace that the images provided will be placed under",
                ),
        )
        .arg(
            Arg::with_name("image")
                .help("Image names to generate the manifest for.")
                .value_name("IMAGE")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("merge")
                .help(
                    "Merge the input specified by --merge-from or STDIN by default with the \
                     output from this invocation.",
                )
                .short("-m")
                .long("merge"),
        )
        .arg(
            Arg::with_name("merge-from")
                .help(
                    "Specify a path other than STDIN to merge from. Use `-` to refer to STDIN.",
                )
                .long("merge-from")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .help(
                    "Specify a path other than STDOUT to output to. Existing files will be \
                     truncated.

                     Use `-` to refer to STDOUT",
                )
                .short("o")
                .long("output")
                .takes_value(true),
        )
}

/// Gets a `Read` depending on the path. If the path is `-`, read from STDIN
fn input_reader(path: &str) -> Result<Box<Read>, String> {
    match path {
        "-" => Ok(Box::new(io::stdin())),
        path => {
            let file = File::open(path).map_err(
                |e| format!("Cannot open input file: {}", e),
            )?;
            Ok(Box::new(file))
        }
    }
}

/// Read from a reader to a String
fn read(mut reader: Box<Read>) -> Result<String, String> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).map_err(|e| {
        format!("Error reading merge input: {}", e)
    })?;

    Ok(buffer)
}

/// Gets a `Write` depending on the path. If the path is `-`, write to STDOUT
fn output_writer(path: &str) -> Result<Box<Write>, String> {
    match path {
        "-" => Ok(Box::new(io::stdout())),
        path => {
            let file = File::create(path).map_err(|e| {
                format!("Cannot open output file: {}", e)
            })?;
            Ok(Box::new(file))
        }
    }
}

/// Serialise a manifest to JSON and write to the writer
fn output<'a>(manifest: &Manifest<'a>, mut writer: Box<Write>) -> Result<(), String> {
    let json = serde_json::to_string_pretty(manifest).map_err(|e| {
        format!("Serialization Error: {}", e)
    })?;
    writer.write_all(json.as_bytes()).map_err(|e| {
        format!("Write error: {}", e)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_parsed_without_tag_returns_latest() {
        let image_name = "hello-world";
        let parsed = Image::parse_image(image_name);
        let expected_parsed = Image {
            image: "hello-world",
            repository: "hello-world",
            tag: "latest",
            tarball: Cow::from("images/hello-world.tar.gz"),
        };

        assert_eq!(expected_parsed, parsed);
    }

    #[test]
    fn image_parsed_with_tag() {
        let image_name = "ubuntu:16.04";
        let parsed = Image::parse_image(image_name);
        let expected_parsed = Image {
            image: "ubuntu:16.04",
            repository: "ubuntu",
            tag: "16.04",
            tarball: Cow::from("images/ubuntu_16.04.tar.gz"),
        };

        assert_eq!(expected_parsed, parsed);
    }
}
