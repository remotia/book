# Introduction

Remotia is a modular, open-source framework for building and benchmarking remote rendering and video streaming pipelines. Designed with cloud gaming as a primary use case, it provides a small set of abstractions — DTOs, processors, components, and pipelines — that compose into directed data-flow graphs. The framework handles channel wiring, task spawning, and lifecycle management so that researchers can focus on experimentation rather than infrastructure. Its platform-agnostic design supports a wide range of codecs, protocols, and hardware configurations, making it suitable both for academic research and practical application [@remotia].

Remotia is an [open-source](https://github.com/remotia) media processing and streaming framework written in [Rust](https://rust-lang.org/).
While being designed with remote rendering and cloud gaming in mind, its components are versatile and can be used in a variety of contexts. 
Bindings and plug-and-play modules for well-established libraries such as [ffmpeg](https://github.com/remotia/remotia-ffmpeg-codecs) and the [SRT protocol](https://github.com/remotia/remotia-srt) are available, while more are in development and will be released soon. See the [API documentation](https://docs.rs/remotia/latest/remotia/) for the full module reference.

The authors are members of the [IPLab](https://iplab.dmi.unict.it/) of the University of Catania.

# Bibliography

If you use remotia in your work, please cite the following paper:

```
@article{remotia,
  title={An open source framework for video streaming in cloud gaming},
  author={Catania, Lorenzo and Giudice, Oliver and Battiato, Sebastiano and Stanco, Filippo and Allegra, Dario},
  journal={Multimedia Tools and Applications},
  pages={1--24},
  year={2025},
  publisher={Springer}
}
```