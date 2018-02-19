# uni-glsl

A just works but completed (GL-ES 2.0 aka WebGL) preprocessor and parser.

## Features

### Preprocessor
Supported syntax : 
```
#
#define
#undef

#ifdef
#ifndef
#else
#endif
#if
#elif
defined
```
Ignored :
```
#line
#version
#extension
#pragma
```
Todo :
```
#include
```

### Parser
Full syntax based on https://www.khronos.org/registry/OpenGL/specs/es/2.0/GLSL_ES_Specification_1.00.pdf

## Usage
See the integeration test in tests/integeration_test.rs

