## About

The check plugin **check\_rungrep** monitors the result of a custom CLI command.

## Build

Compile like any other Rust program: `cargo build -r`

Find the resulting binary directly under `target/release/`.

## Usage

### `command EXE [ARGS...]`

```
$ check_rungrep \
  command cat /dev/null
$ echo $?
0
```

In this minimal working example,
`check_rungrep` will just invoke `cat /dev/null`.
While the "command" parameter is required,
it almost doesn't do anything useful on its own.

`check_rungrep` will only complain if it fails to exec(3)/waitpid(2)
the given executable or the latter is terminated by a signal.

```
$ check_rungrep command dog
☯️ exec(3): No such file or directory (os error 2)
$ echo $?
3
```

```
$ check_rungrep command bash -c 'kill -9 $$'
☯️ waitpid(2): child was killed
$ echo $?
3
```

In general, "command" should be combined with any number of other parameters
which are described below. Each one can be specified any number of times.

In addition, if the environment variable `CHECK_RUNGREP_STDIN` is set,
its value is written to stdin of the spawned process.

```
$ CHECK_RUNGREP_STDIN=secret check_rungrep command cat
$ echo $?
0
```

### `cd DIR`

```
$ check_rungrep \
  cd /dev \
  command cat null
$ echo $?
0
```

Change the working directory before running any program, complain on failure.

```
$ check_rungrep \
  cd /nosuchdir \
  command cat
☯️ chdir(2): No such file or directory (os error 2)
$ echo $?
3
```

### `time WARN CRIT LABEL`

```
$ check_rungrep \
  time 0.003 1 run_seconds \
  command cat /dev/null
⚠️ Command ran for 0.003664692 seconds (3ms 664us 692ns). Warning: 0:0.003. Critical: 0:1.
 | 'run_seconds'=0.003664692s;0:0.003;0:1;0;
...
$ echo $?
1
```

Complain if the command runs for more than [WARN/CRIT] seconds
and/or report the execution time machine-readably using LABEL.
Any of WARN CRIT LABEL may be empty strings for no-op.

```
$ check_rungrep \
  time '' '' '' \
  command cat /dev/null
✅ Command ran for 0.002486996 seconds (2ms 486us 996ns).
$ echo $?
0
```

### `exit WARN CRIT LABEL`

```
$ check_rungrep \
  exit 0 1 return_code \
  command cat /dev/null
✅ Command returned 0. Warning: 0:0. Critical: 0:1.
 | 'return_code'=0;0:0;0:1;;
$ echo $?
0
```

Complain if the command returns a code out of [WARN/CRIT] range
and/or report the exit code machine-readably using LABEL.
Any of WARN CRIT LABEL may be empty strings for no-op.

```
$ check_rungrep \
  exit '' '' '' \
  command cat /dev/null
✅ Command returned 0.
$ echo $?
0
```

### `stdout|stderr literal|regex PATTERN WARN CRIT LABEL`

```
# check_rungrep \
  stdout literal "pool 'zroot' is healthy" '' 1:1 '' \
  command zpool status -x zroot
✅ Command's stdout matched the following pattern 1 times. Critical: 1:1. Literal string: pool 'zroot' is healthy
# echo $?
0
```

Complain if the command's stdout/stderr matches PATTERN more/less often
than [WARN/CRIT] and/or report the matches count machine-readably using LABEL.
Any of WARN CRIT LABEL may be empty strings for no-op.
Instead of running as root, consider doas(1) or sudo(8).

```
$ check_rungrep \
  stdout literal "pool 'zroot' is healthy" '' 1:1 '' \
  command doas zpool status -x zroot
✅ Command's stdout matched the following pattern 1 times. Critical: 1:1. Literal string: pool 'zroot' is healthy
$ echo $?
0
```


[WARN/CRIT]: https://nagios-plugins.org/doc/guidelines.html#THRESHOLDFORMAT
