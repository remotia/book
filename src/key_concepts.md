# Key concepts

This section introduces the fundamental concepts of the framework, designed to facilitate the creation of video streaming processing pipelines.
The content of this page is mostly a resume of the chapter *3.1 Basic Concepts* of the [original paper](https://link.springer.com/article/10.1007/s11042-025-20798-y), updated in line with the latest version of the framework.
These concepts form the foundation of the framework, providing a modular and flexible system for designing and implementing video streaming architectures. 

## Data Transfer Object (DTO)

<img style="display:block; margin: auto" src="./figures/frame_dto.svg">

The framework uses a **Data Transfer Object (DTO)** to handle the transfer of information between components. 
This object is referred to as Frame DTO or simply DTO.
It usually contains pointers to data buffers and statistical values related to the frame(s) being processed. 

The structure of the DTO is intentionally kept generic
Each application should define its own DTO types adapted to its specific use cases.
A DTO implements the necessary interfaces [traits](https://doc.rust-lang.org/book/ch10-02-traits.html) to interact with processors. 
In most cases, the DTOs implement interfaces that are part of the standard library
This paradigm ensures that data are decoupled from the the logic code, which remains reusable.
For more complex scenarios, custom modules can define their own interfaces that the DTO must implement if the module is used in the pipeline. 

## Processors
<img style="display:block; margin: auto; width: 30em" src="./figures/processor.svg">

A **processor** represents a single unit of operations applied to a DTO. 
Its purpose is to simplify the process of modifying or analyzing atomic steps of streaming pipelines. 
During its lifecycle, a DTO is passed through a sequence of processors, each of which may read or modify the data.

Each processor takes one DTO as input and produces one or zero DTOs as output. 
The output DTO can either be the modified input DTO or a newly allocated one. 
If an output DTO is produced, it is passed to the next processor in the sequence. 
This flow is analogous to the layers in a neural network, where each processor performs a specific task.

Processors have full [ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) of the Frame DTO once they receive it. 
If a processor does not produce an output, it signals that the current processor must hold the DTO outside the current execution cycle or that the processing should be transferred to a different pipeline or suspended.

## Components

<img style="display:block; margin: auto" src="./figures/component.svg">

A **component** is the context in which a sequence of processors is executed. 
Each component runs asynchronously with the rest of the system, processes the data it receives, and sends the resulting data to another connected component. 
Components can also periodically allocate an empty DTO, fill it with initial data, and send it through the architecture.

In practical terms, a component corresponds to a thread executing a sequence of processors. 
The framework uses a pattern known as [green threads](https://tokio.rs/tokio/tutorial/spawning#tasks), reducing the overhead of asynchronous executions. 
Grouping processors into components allows to leverage the multi-threading capabilities of modern CPUs. 
This design helps modifying the load distribution with minimal modifications to the code.

## Pipelines

<img style="display:block; margin: auto" src="./figures/pipeline.svg">

A **pipeline** is a sequentially connected set of components that share a common scope and can work asynchronously. 
Pipelines are used to link different components that process frames in distinct ways, such as handling errors or profiling.

While a simple architecture may consist of a single pipeline, more complex systems often require multiple pipelines to handle different processing contexts. 
For example, a pipeline might be dedicated to logging or debugging, while another handles the main streaming process.

### Switches
Processors that can move data between pipelines are referred to as **switches**. 
These components are essential for scaling systems to support multi-user streaming or more complex scenarios involving multiple frame data sources and sinks. 
