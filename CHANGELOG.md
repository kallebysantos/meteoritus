# Version 0.2.0 (May 3, 2023)

## Major Features and Improvements

* Added [Termination extension](https://tus.io/protocols/resumable-upload.html#termination) functionality.

    This extension defines a way for the Client to terminate completed and unfinished uploads allowing the Server to free up used resources.
    Available from [`on_termination()`](https://docs.rs/meteoritus/0.2.0/meteoritus/struct.Meteoritus.html#method.on_termination) option.

* Added [`keep_on_disk()`](https://docs.rs/meteoritus/0.2.0/meteoritus/struct.Meteoritus.html#method.keep_on_disk) option.

    From now all completed uploads resources will be deleted from Server, this option specifies that theses temp files should be kept on disk.

## General Improvements

  * Features decorated with `#![feature(...)]` has refactored to `stable` code.

## Infrastructure
  * GitHub CI workflow creation to improve code quality.

# Version 0.1.0 (Apr 7, 2023)

## Library was been published to crates.io
