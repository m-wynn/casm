[![Build Status](https://travis-ci.org/m-wynn/casm.svg?branch=master)](https://travis-ci.org/m-wynn/casm)
[![codecov](https://codecov.io/gh/m-wynn/casm/branch/master/graph/badge.svg)](https://codecov.io/gh/m-wynn/casm)
[![license](https://img.shields.io/badge/license-ISC-blue.svg)](https://github.com/m-wynn/casm/blob/master/LICENSE)
[![docs](https://img.shields.io/badge/docs-m--wynn.github.io%2Fcasm-orange.svg)](https://m-wynn.github.io/casm)


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
1. Read command-line options for specifying a custom configuration file
   location, or overriding options in the config file.
1. Scan music in the paths I specified using some sort of globbing or regex.
1. Ignore a set of files (i.e. instrumental versions) using globbing or regex.
1. Determine if there is a newer valid copy in the destination.
1. Determine if the music should be converted, or simply copied.
1. Convert and copy in multiple threads.
