# manifest

A CLI tool to convert a bunch of Docker Image names, along with their namespace into some serializable format
(e.g. JSON) containing their components and other information that can then be used by other tools such
as a deployment tool.

Specifically, in the case of our current projects, this tool will turn Docker image names
along with the application names into a JSON format that can then be parsed by Ansible to deploy applications.

Currently, there is no way to specify "additional information". This is a `TODO` for future extension.

## Example output

```json
{
  "app_one": [
    {
      "repository": "my.private.registry/app_one",
      "tag": "2017-07-06_07-31-20-554b02f",
      "image": "my.private.registry/app_one:2017-07-06_07-31-20-554b02f",
      "tarball": "images/my.private.registry_app_one_2017-07-06_07-31-20-554b02f.tar.gz"
    },
    {
      "repository": "mdillon/postgis",
      "tag": "9.6",
      "image": "mdillon/postgis:9.6",
      "tarball": "images/mdillon_postgis_9.6.tar.gz"
    }
  ],
  "app_two": [
    {
      "repository": "my.private.registry/app_two",
      "tag": "2017-07-06_07-30-57-98cd17f",
      "image": "my.private.registry/app_two:2017-07-06_07-30-57-98cd17f",
      "tarball": "images/my.private.registry_app_two_2017-07-06_07-30-57-98cd17f.tar.gz"
    }
  ],
  "app_three": [
    {
      "repository": "my.private.registry/app_three",
      "tag": "2017-07-06_07-31-49-f94bd15",
      "image": "my.private.registry/app_three:2017-07-06_07-31-49-f94bd15",
      "tarball": "images/my.private.registry_app_three_2017-07-06_07-31-49-f94bd15.tar.gz"
    },
    {
      "repository": "postgres",
      "tag": "9.5",
      "image": "postgres:9.5",
      "tarball": "images/postgres_9.5.tar.gz"
    }
  ],
  "app_three": [
    {
      "repository": "my.private.registry/app_three",
      "tag": "2017-07-06_07-32-27-fc94297",
      "image": "my.private.registry/app_three:2017-07-06_07-32-27-fc94297",
      "tarball": "images/my.private.registry_app_three_2017-07-06_07-32-27-fc94297.tar.gz"
    },
    {
      "repository": "postgres",
      "tag": "9.5",
      "image": "postgres:9.5",
      "tarball": "images/postgres_9.5.tar.gz"
    }
  ],
  "utils": [
    {
      "repository": "lawliet89/rowdy",
      "tag": "v0.0.4",
      "image": "lawliet89/rowdy:v0.0.4",
      "tarball": "images/lawliet89_rowdy_v0.0.4.tar.gz"
    },
    {
      "repository": "gyng/rcanary",
      "tag": "latest",
      "image": "gyng/rcanary:latest",
      "tarball": "images/gyng_rcanaryatest.tar.gz"
    },
    {
      "repository": "lawliet89/map-tiles-gl",
      "tag": "v1.7.0",
      "image": "lawliet89/map-tiles-gl:v1.7.0",
      "tarball": "images/lawliet89_map-tiles-gl_v1.7.0.tar.gz"
    },
    {
      "repository": "nginx",
      "tag": "alpine",
      "image": "nginx:alpine",
      "tarball": "images/nginx_alpine.tar.gz"
    },
    {
      "repository": "jwilder/docker-gen",
      "tag": "0.7.2",
      "image": "jwilder/docker-gen:0.7.2",
      "tarball": "images/jwilder_docker-gen_0.7.2.tar.gz"
    }
  ]
}
```

## Usage

If you notice, in the output above, the top level JSON object is a dictionary with several keys. In the
context of the CLI, the top level keys are called "namespace".

Due to the limitation of the `clap` command line argument parser, there is no way to enforce order-dependent
parsing of command line options and position based arguments. Thus, each invocation can only support one namespace.

To merge multiple namespaces into one single JSON output, simply pipe the invocations and then use the `--merge`
flag to merge inputs with that of STDIN, or another file. See the example below.

### Example Invocation and Output

```bash
$ manifest --namespace utils gyng/rcanary:latest jwilder/docker-gen:0.7.2 \
  | manifest --namespace app_one my.private.registry/app_one:2017-07-06_07-31-20-554b02f mdillon/postgis:9.6 --merge
```

```json
{
  "app_one": [
    {
      "image": "my.private.registry/app_one:2017-07-06_07-31-20-554b02f",
      "repository": "my.private.registry/app_one",
      "tag": "2017-07-06_07-31-20-554b02f",
      "tarball": "images/my.private.registry_app_one_2017-07-06_07-31-20-554b02f.tar.gz"
    },
    {
      "image": "mdillon/postgis:9.6",
      "repository": "mdillon/postgis",
      "tag": "9.6",
      "tarball": "images/mdillon_postgis_9.6.tar.gz"
    }
  ],
  "utils": [
    {
      "image": "gyng/rcanary:latest",
      "repository": "gyng/rcanary",
      "tag": "latest",
      "tarball": "images/gyng_rcanaryatest.tar.gz"
    },
    {
      "image": "jwilder/docker-gen:0.7.2",
      "repository": "jwilder/docker-gen",
      "tag": "0.7.2",
      "tarball": "images/jwilder_docker-gen_0.7.2.tar.gz"
    }
  ]
}
```

### CLI argumments, flags, and options

```txt
manifest 0.1.0
Yong Wen Chua <me@yongwen.xyz>
Generate a serialized manifest of Docker Image names, components and additonal values from a list of Docker Images

Each invocation of the CLI allows one single namespace. You can pipe this to a subsequent invocation of the CLI tool with the `--merge` flag to ask the CLI to
merge the output with whatever is fed into it from either the STDIN or the path specified by --merge-from.

If no `--output` is specified, the CLI will write to STDOUT.

USAGE:
    manifest [FLAGS] [OPTIONS] <IMAGE>... --namespace <namespace>

FLAGS:
    -h, --help       Prints help information
    -m, --merge      Merge the input specified by --merge-from or STDIN by default with the output from this invocation.
    -V, --version    Prints version information

OPTIONS:
        --merge-from <merge-from>    Specify a path other than STDIN to merge from. Use `-` to refer to STDIN.
    -n, --namespace <namespace>      The namespace that the images provided will be placed under
    -o, --output <output>            Specify a path other than STDOUT to output to. Existing files will be truncated.

                                     Use `-` to refer to STDOUT

ARGS:
    <IMAGE>...    Image names to generate the manifest for.
```
