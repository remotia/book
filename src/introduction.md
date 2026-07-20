# Introduction

Remotia is a modular, open-source framework for building and benchmarking remote rendering and video streaming pipelines. Designed with cloud gaming as a primary use case, it provides a small set of abstractions — DTOs, processors, components, and pipelines — that compose into sequential processing chains. The framework links the building blocks and handles their execution so that researchers can focus on experimentation rather than infrastructure. Its platform-agnostic design supports any ffmpeg-compliant codec and protocol, with no hard dependencies on operating systems or proprietary protocols, making it suitable both for academic research and practical application [@remotia].

Remotia is an [open-source](https://github.com/remotia) media processing and streaming framework written in [Rust](https://rust-lang.org/).
While being designed with remote rendering and cloud gaming in mind, its components are versatile and can be used in a variety of contexts. 
Bindings and plug-and-play modules for well-established libraries such as [ffmpeg](https://github.com/remotia/remotia-ffmpeg-codecs) and the [SRT protocol](https://github.com/remotia/remotia-srt) are available, while ready-to-plug-in components for audio streaming and user input handling are in development; custom modules can already be developed to implement additional features if needed. See the [API documentation](https://docs.rs/remotia/latest/remotia/) for the full module reference.

Four of the five authors are with the [IPLab](https://iplab.dmi.unict.it/) of the University of Catania; the remaining author is with the Bank of Italy.

# Bibliography

If you use remotia in your work, please cite the following paper:

```
@article{remotia,
  title={An open source framework for video streaming in cloud gaming},
  author={Catania, Lorenzo and Giudice, Oliver and Battiato, Sebastiano and Stanco, Filippo and Allegra, Dario},
  journal={Multimedia Tools and Applications},
  volume={84},
  pages={41381--41404},
  year={2025},
  doi={10.1007/s11042-025-20798-y},
  publisher={Springer}
}
```