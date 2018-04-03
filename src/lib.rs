//! This crate provides a graphical implementation of Conway's Game of Life.
//!
//! It uses OpenGL and GLFW for the GUI and a multi-threaded backend for the
//! simulation.

#![warn(missing_docs, trivial_numeric_casts, unused_extern_crates,
        unused_qualifications, unused_results)]

pub mod gui;
pub mod backend;
