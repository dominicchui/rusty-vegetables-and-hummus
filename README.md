# Rusty Vegetables and Hummus

## Ecological Simulation and Procedural Modeling
This is an implementation of the paper "Authoring Landscapes by Combining Ecosystem and Terrain Erosion Simulation" by Cordonnier, G. et al.

![sample_7-100](https://github.com/dominicchui/rusty-vegetables-and-hummus/assets/36180122/531e4994-6379-4231-9daa-66e16de6d833)

The core idea of this paper is to simulate the complex interplay between geomorphology, terrain, and vegetation. The main challenge with Euler step based approaches is that the step size is tied to the quickest phenomena. When it comes to ecology, the time scale differences between event like lighting strikes and erosion make it untenable. The alternative is an event based approach, where discrete events representing different phenomena. 

### Framework

The design of this system is composed of two parts: the discretized terrain grid and the stochastic events. The physical space is dividied into uniform columnar grid cells and each grid cell is composed of different layers.

<img width="440" alt="Screenshot 2024-08-27 at 11 54 19 PM" src="https://github.com/user-attachments/assets/50ac7989-e0a8-4955-a73d-48dcd38ede0d">

<img width="420" alt="Screenshot 2024-08-27 at 11 54 38 PM" src="https://github.com/user-attachments/assets/e9b4cfa0-bc62-4ef4-8b3f-78a986ec8016">

In each time step of the simulation, N*M events are randomly chosen, where N is the number of grid cells and M is the number of events, making sure that each event occurs once in each cell. Events can impact the cell itself, neighboring cells, and also propagate. We implemented a total of four geomorphological events: rainfall (and running water), lightning, thermal erosion, and rock/sand/humus fall. We also implemented the complex dynamics of vegetation growth and death, where vigor and stress are computed independently for trees, bushes, and grasses based on the soil moisture, temperature, and sun exposure, which come from rainfall, terrain height, and ray tracing the averaged position of the sun, respectively.

### Specifics

The simulation is run in real time and rendered using OpenGL. The initial terrain is generated based on real world height-maps. This implementation is full of magic numbers and constants and most are obtained from the literature, while some were selected for their simulation-friendliness.

## Extension
For our extension, we integrated sand dunes simultion from the paper "Desert Simulation” by Paris, A. et al. This paper simulates wind over a discretized grid to procedurally generate realistic sand dunes.

<img width="739" alt="Screenshot 2024-05-14 at 5 06 33 PM" src="https://github.com/dominicchui/rusty-vegetables-and-hummus/assets/36180122/a99d6f0a-9051-498b-b653-f1ece4e5aeb8">

The core ideas is that multiple wind phenomena will be modeled and combined. First a wind rose will model the high altitude wind field. Each simulation step, a direction and magnitude will be randomly chosen from the predefined wind rose and applied to the entire terrain. This wind will then be warped by local terrain, which was determined using a high and low resolution convolution with a gaussian kernel, to model how wind contours around elevated terrain. Finally, on the grid cell level, wind shadowing is modeled where the lee side of elevated terrain experiences decreased wind speed. The simple wind model uses just the wind rose but the full model uses all three.

In terms of how the wind event affects individual grid cells, sand is moved in three steps. 1) Saltation: sand is lifted from a cell, carried with the wind, and bounces on other cells, eventually becoming deposited. 2) Reptation: when sand bounces on a cell, sand already in the cell is collided with and is moved to adjacent cells. 3) Avalanching: this is the same event as sand fall, namely that when the sand angle is higher than the angle of repose, it slides.

<img width="582" alt="Screenshot 2024-08-28 at 12 05 29 AM" src="https://github.com/user-attachments/assets/15c4aa7e-f9ab-4a39-ad1f-26a5128b2feb">

This paper integrated very cleanly with the original since they both relied on the framework of the discretized grid and stochastic events. The events also interacted with each nicely, as vegetation increases the likelihood that sand will be deposited and is simulated by the vegetation events in the base paper. Wind erosion is also, unsurprisingly, strikingly similar to erosion from rainfall.

### Results

Simple wind model

https://github.com/dominicchui/rusty-vegetables-and-hummus/assets/36180122/944fd5d0-20fd-4e60-abe5-dbf8e35838c5

Simple wind model with obstacles

https://github.com/dominicchui/rusty-vegetables-and-hummus/assets/36180122/c2e915ff-2d69-4569-bbd4-12259b5cf13d

Complex wind model

https://github.com/dominicchui/rusty-vegetables-and-hummus/assets/36180122/703739d9-1504-4a15-a61e-b06ea94e8d27

