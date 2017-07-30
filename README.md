[![Build Status](https://travis-ci.org/m-wynn/casm.svg?branch=master)](https://travis-ci.org/m-wynn/casm)

CASM
====

Convert and Sync Music

I have a lot of music in various lossless and lossy formats.  I would like to
be able to convert the lossless ones to lossy ones and sync them to another
folder (usually my phone).  I would like to do this in a multithreaded way.

This also serves to help me learn Rust better.  I wrote a
[thing](https://github.com/m-wynn/cacm) in bash, but bash arrays are slow, and
I would like it to do more.

Objectives
----------

This software should be able to:

1. Read a configuration file with options on what and how to sync.
2. Read command-line options for specifying a custom configuration file
   location, or overriding options in the config file.
3. Scan music in the paths I specified using some sort of globbing or regex.
4. Determine if there is a newer valid copy in the destination.
5. Determine if the music should be converted, or simply copied.
6. Convert and copy in multiple threads.
